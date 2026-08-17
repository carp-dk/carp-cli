// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Downloading from the command line.
//!
//! `carp_client::transfer` does the streaming; this decides what a person
//! watching it should see. Progress goes to stderr and only when a terminal is
//! there to redraw it, so `carp export download … | tar -x` is not fed a
//! progress bar, and a CI log does not fill with one.

use std::io::{self, IsTerminal, Write};

use carp_client::api::models::format_bytes;
use color_eyre::Result;
use serde::Serialize;

use crate::commands::Session;
use crate::output;

/// Where a download landed, for `--format json`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Downloaded {
    path: String,
    bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    study_id: Option<String>,
}

/// Stream `api_path` into the configured download directory.
pub async fn to_disk(
    session: &Session,
    api_path: &str,
    fallback_name: &str,
    study_id: Option<String>,
) -> Result<()> {
    let show_progress = io::stderr().is_terminal();
    let mut last_shown = 0_u64;

    let transfer = carp_client::transfer::download(
        &session.client,
        api_path,
        &session.config.download_dir,
        fallback_name,
        |received, total| {
            if !show_progress {
                return;
            }
            // Redraw in place. Rate-limited by the library already, but a
            // stalled transfer would otherwise repaint the same line forever.
            if received == last_shown && received > 0 {
                return;
            }
            last_shown = received;
            let mut stderr = io::stderr();
            let _ = match total {
                Some(total) if total > 0 => write!(
                    stderr,
                    "\r{fallback_name}: {} / {} ({:.0}%)  ",
                    format_bytes(received),
                    format_bytes(total),
                    (received as f64 / total as f64) * 100.0
                ),
                _ => write!(stderr, "\r{fallback_name}: {}  ", format_bytes(received)),
            };
            let _ = stderr.flush();
        },
    )
    .await?;

    if show_progress {
        // Clear the progress line so it does not sit half-written above the
        // result.
        let _ = write!(io::stderr(), "\r\u{1b}[2K");
        let _ = io::stderr().flush();
    }

    let downloaded = Downloaded {
        path: transfer.path.display().to_string(),
        bytes: transfer.bytes,
        study_id,
    };
    let lines = vec![
        ("saved", downloaded.path.clone()),
        ("size", format_bytes(downloaded.bytes)),
    ];
    output::detail(&downloaded, &lines, session.format)
}
