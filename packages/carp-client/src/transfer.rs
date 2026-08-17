// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Streaming a file or export to disk.
//!
//! Exports are large enough that buffering one whole would be felt, so the
//! response is written as it arrives. Progress is reported through a closure
//! rather than a channel: a terminal draws a bar from it, a script prints
//! nothing, and neither has to know about the other's plumbing.

use std::path::{Path, PathBuf};

use futures_util::StreamExt;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::api::CarpClient;
use crate::api::error::{ApiError, ApiResult};

/// Report roughly every quarter mebibyte rather than per chunk: a progress
/// callback that fires thousands of times a second costs more than it tells.
const PROGRESS_INTERVAL: u64 = 256 * 1024;

/// Where a finished transfer landed, and how much of it there was.
#[derive(Debug, Clone)]
pub struct Transfer {
    pub path: PathBuf,
    pub bytes: u64,
}

/// Stream `api_path` into `directory`, calling `progress` as it goes.
///
/// The file name comes from `Content-Disposition` when the server provides one
/// and falls back to `fallback_name`. An existing file is never overwritten.
///
/// `progress` receives the bytes received so far and the total when the server
/// declared one — it often does not, for a response it is still generating.
pub async fn download(
    client: &CarpClient,
    api_path: &str,
    directory: &Path,
    fallback_name: &str,
    mut progress: impl FnMut(u64, Option<u64>),
) -> ApiResult<Transfer> {
    let response = client.get_stream(api_path).await?;
    let total = response.content_length();
    let name = content_disposition_name(&response).unwrap_or_else(|| fallback_name.to_owned());

    fs::create_dir_all(directory).await?;
    let path = unique_path(directory, &sanitize(&name)).await;

    progress(0, total);

    let mut file = fs::File::create(&path).await?;
    let mut stream = response.bytes_stream();
    let mut received = 0_u64;
    let mut last_report = 0_u64;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(ApiError::from)?;
        file.write_all(&chunk).await?;
        received += chunk.len() as u64;
        if received - last_report >= PROGRESS_INTERVAL {
            last_report = received;
            progress(received, total);
        }
    }
    file.flush().await?;
    progress(received, total);

    Ok(Transfer {
        path,
        bytes: received,
    })
}

/// Read the file name out of a `Content-Disposition` header.
pub fn content_disposition_name(response: &reqwest::Response) -> Option<String> {
    let header = response
        .headers()
        .get(reqwest::header::CONTENT_DISPOSITION)?
        .to_str()
        .ok()?;
    for part in header.split(';') {
        let part = part.trim();
        if let Some(value) = part.strip_prefix("filename=") {
            let value = value.trim_matches('"').trim();
            if !value.is_empty() {
                return Some(value.to_owned());
            }
        }
    }
    None
}

/// Keep the name a single, harmless path segment.
pub fn sanitize(name: &str) -> String {
    let name: String = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '\0' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();
    let name = name.trim().trim_matches('.').to_owned();
    if name.is_empty() {
        "carp-download".to_owned()
    } else {
        name
    }
}

/// Never overwrite an existing download: `report.zip` becomes `report (2).zip`.
pub async fn unique_path(directory: &Path, name: &str) -> PathBuf {
    let candidate = directory.join(name);
    if !candidate.exists() {
        return candidate;
    }
    let path = Path::new(name);
    let stem = path
        .file_stem()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| name.to_owned());
    let extension = path
        .extension()
        .map(|value| format!(".{}", value.to_string_lossy()))
        .unwrap_or_default();

    for index in 2..1000 {
        let candidate = directory.join(format!("{stem} ({index}){extension}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    // Suffix of last resort when a thousand copies already exist.
    directory.join(format!(
        "{stem}-{}{extension}",
        chrono::Utc::now().timestamp()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `Content-Disposition` comes from the server, so it is not trusted to
    /// be a bare file name. What has to hold is that the result cannot leave
    /// the directory it is joined to, whatever arrives.
    #[test]
    fn a_name_is_reduced_to_one_harmless_segment() {
        assert_eq!(sanitize("report.zip"), "report.zip");
        assert_eq!(sanitize("a/b/c.zip"), "a_b_c.zip");
        assert_eq!(sanitize("  spaced.zip  "), "spaced.zip");
        assert_eq!(sanitize("a\u{0}b"), "a_b");

        for hostile in [
            "../../etc/passwd",
            "..\\..\\etc\\passwd",
            "/etc/passwd",
            "C:\\Windows\\System32",
            "..",
        ] {
            let safe = sanitize(hostile);
            assert_eq!(
                Path::new(&safe).components().count(),
                1,
                "{hostile} sanitised to {safe}, which is not one component"
            );
            assert!(
                !safe.contains(['/', '\\']),
                "{hostile} sanitised to {safe}, which still separates"
            );
            assert_ne!(safe, "..", "{hostile} sanitised to a parent reference");
        }
    }

    /// A server that sends only dots, or nothing at all, must still produce a
    /// file rather than an empty path that resolves to the directory itself.
    #[test]
    fn an_empty_name_still_names_something() {
        assert_eq!(sanitize(""), "carp-download");
        assert_eq!(sanitize("..."), "carp-download");
        assert_eq!(sanitize("   "), "carp-download");
    }

    #[tokio::test]
    async fn an_existing_file_is_never_overwritten() {
        let directory = std::env::temp_dir().join("carp-transfer-unique-test");
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).unwrap();

        assert_eq!(
            unique_path(&directory, "report.zip").await,
            directory.join("report.zip")
        );

        std::fs::write(directory.join("report.zip"), b"first").unwrap();
        assert_eq!(
            unique_path(&directory, "report.zip").await,
            directory.join("report (2).zip")
        );

        std::fs::write(directory.join("report (2).zip"), b"second").unwrap();
        assert_eq!(
            unique_path(&directory, "report.zip").await,
            directory.join("report (3).zip")
        );

        // A name without an extension keeps its shape too.
        std::fs::write(directory.join("notes"), b"x").unwrap();
        assert_eq!(
            unique_path(&directory, "notes").await,
            directory.join("notes (2)")
        );

        std::fs::remove_dir_all(&directory).unwrap();
    }
}
