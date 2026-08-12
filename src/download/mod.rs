// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Download queue.
//!
//! [`DownloadManager`] is plain state owned by the app; the actual transfer
//! runs in a background task ([`run`]) that streams the response to disk and
//! reports progress as messages.

use std::path::{Path, PathBuf};

use futures_util::StreamExt;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::UnboundedSender;

use crate::api::CarpClient;
use crate::api::error::{ApiError, ApiResult};
use crate::api::models::format_bytes;
use crate::app::message::Message;

/// Identifies a queued transfer.
pub type JobId = u64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobStatus {
    Running,
    Done,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct DownloadJob {
    pub id: JobId,
    /// What is being downloaded, e.g. the export or file name.
    pub label: String,
    /// Study the download belongs to, if any.
    pub study_id: Option<String>,
    pub status: JobStatus,
    pub received: u64,
    pub total: Option<u64>,
    pub path: Option<PathBuf>,
}

impl DownloadJob {
    /// Progress in `0.0..=1.0`; `None` when the server sends no length.
    pub fn ratio(&self) -> Option<f64> {
        let total = self.total?;
        if total == 0 {
            return None;
        }
        Some((self.received as f64 / total as f64).clamp(0.0, 1.0))
    }

    pub fn progress_label(&self) -> String {
        match (&self.status, self.total) {
            (JobStatus::Failed(error), _) => error.clone(),
            (JobStatus::Done, _) => format!("{} done", format_bytes(self.received)),
            (JobStatus::Running, Some(total)) => {
                format!("{} / {}", format_bytes(self.received), format_bytes(total))
            }
            (JobStatus::Running, None) => format_bytes(self.received),
        }
    }

    pub fn is_finished(&self) -> bool {
        !matches!(self.status, JobStatus::Running)
    }
}

#[derive(Debug, Default)]
pub struct DownloadManager {
    jobs: Vec<DownloadJob>,
    next_id: JobId,
}

impl DownloadManager {
    /// Register a transfer and return its id.
    pub fn enqueue(&mut self, label: String, study_id: Option<String>) -> JobId {
        self.next_id += 1;
        let id = self.next_id;
        self.jobs.insert(
            0,
            DownloadJob {
                id,
                label,
                study_id,
                status: JobStatus::Running,
                received: 0,
                total: None,
                path: None,
            },
        );
        id
    }

    pub fn jobs(&self) -> &[DownloadJob] {
        &self.jobs
    }

    pub fn active_count(&self) -> usize {
        self.jobs
            .iter()
            .filter(|job| job.status == JobStatus::Running)
            .count()
    }

    pub fn is_empty(&self) -> bool {
        self.jobs.is_empty()
    }

    pub fn progress(&mut self, id: JobId, received: u64, total: Option<u64>) {
        if let Some(job) = self.job_mut(id) {
            job.received = received;
            if total.is_some() {
                job.total = total;
            }
        }
    }

    pub fn finish(&mut self, id: JobId, path: PathBuf, bytes: u64) {
        if let Some(job) = self.job_mut(id) {
            job.status = JobStatus::Done;
            job.received = bytes;
            job.path = Some(path);
        }
    }

    pub fn fail(&mut self, id: JobId, error: String) {
        if let Some(job) = self.job_mut(id) {
            job.status = JobStatus::Failed(error);
        }
    }

    /// Drop everything that is no longer running.
    pub fn clear_finished(&mut self) {
        self.jobs.retain(|job| !job.is_finished());
    }

    fn job_mut(&mut self, id: JobId) -> Option<&mut DownloadJob> {
        self.jobs.iter_mut().find(|job| job.id == id)
    }
}

/// Stream `api_path` into `directory`, reporting progress on `tx`.
///
/// The file name comes from `Content-Disposition` when the server provides one
/// and falls back to `fallback_name`.
pub async fn run(
    client: CarpClient,
    api_path: String,
    directory: PathBuf,
    fallback_name: String,
    job_id: JobId,
    tx: UnboundedSender<Message>,
) {
    let result = transfer(&client, &api_path, &directory, &fallback_name, job_id, &tx).await;
    let message = match result {
        Ok((path, bytes)) => Message::DownloadFinished {
            job_id,
            path,
            bytes,
        },
        Err(error) => Message::DownloadFailed {
            job_id,
            error: error.to_string(),
        },
    };
    let _ = tx.send(message);
}

async fn transfer(
    client: &CarpClient,
    api_path: &str,
    directory: &Path,
    fallback_name: &str,
    job_id: JobId,
    tx: &UnboundedSender<Message>,
) -> ApiResult<(PathBuf, u64)> {
    let response = client.get_stream(api_path).await?;
    let total = response.content_length();
    let name = content_disposition_name(&response).unwrap_or_else(|| fallback_name.to_owned());

    fs::create_dir_all(directory).await?;
    let destination = unique_path(directory, &sanitize(&name)).await;

    let _ = tx.send(Message::DownloadProgress {
        job_id,
        received: 0,
        total,
    });

    let mut file = fs::File::create(&destination).await?;
    let mut stream = response.bytes_stream();
    let mut received = 0_u64;
    let mut last_report = 0_u64;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(ApiError::from)?;
        file.write_all(&chunk).await?;
        received += chunk.len() as u64;
        // Report roughly every 256 KiB rather than per chunk.
        if received - last_report >= 256 * 1024 {
            last_report = received;
            let _ = tx.send(Message::DownloadProgress {
                job_id,
                received,
                total,
            });
        }
    }
    file.flush().await?;

    Ok((destination, received))
}

/// Read the file name out of a `Content-Disposition` header.
fn content_disposition_name(response: &reqwest::Response) -> Option<String> {
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
fn sanitize(name: &str) -> String {
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
async fn unique_path(directory: &Path, name: &str) -> PathBuf {
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
