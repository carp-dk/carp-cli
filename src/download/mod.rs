// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Download queue.
//!
//! [`DownloadManager`] is plain state owned by the app; the transfer itself is
//! `carp_client::transfer`, run in a background task ([`run`]) that turns its
//! progress callback into messages the render loop can act on.

use std::path::PathBuf;

use tokio::sync::mpsc::UnboundedSender;

use carp_client::api::CarpClient;
use carp_client::api::models::format_bytes;

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
    let progress = |received, total| {
        let _ = tx.send(Message::DownloadProgress {
            job_id,
            received,
            total,
        });
    };

    let message = match carp_client::transfer::download(
        &client,
        &api_path,
        &directory,
        &fallback_name,
        progress,
    )
    .await
    {
        Ok(transfer) => Message::DownloadFinished {
            job_id,
            path: transfer.path,
            bytes: transfer.bytes,
        },
        Err(error) => Message::DownloadFailed {
            job_id,
            error: error.to_string(),
        },
    };
    let _ = tx.send(message);
}
