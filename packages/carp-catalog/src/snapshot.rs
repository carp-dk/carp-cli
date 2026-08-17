// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! The downloaded protocols, on disk.
//!
//! A snapshot is what a sync leaves behind: the upstream commit, and every
//! study protocol as it was at that commit. Keeping the documents rather than
//! only the derived vocabulary means two things the editor needs:
//!
//! - the studies can be offered as **templates**, so a new protocol can start
//!   from the demo study rather than from nothing
//! - the vocabulary can be **re-derived** without another download, which
//!   matters when this tool learns to extract something it previously ignored
//!
//! The file is plain JSON in the CLI's data directory, written whole. It is a
//! cache: losing it costs a re-sync and nothing else.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::source::{Commit, ProtocolDocument};
use crate::{Error, Result};

/// Layout version of the snapshot file.
///
/// Bumped whenever the stored shape changes. A snapshot written under a
/// different version is discarded rather than misread, which costs a re-sync
/// and avoids a build silently misunderstanding an older cache.
pub const FORMAT_VERSION: u32 = 1;

/// File name of the snapshot within the data directory.
const FILE_NAME: &str = "protocol-catalog.json";

/// Everything one sync produced.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    /// Layout version; see [`FORMAT_VERSION`].
    pub format_version: u32,
    /// `owner/repo` the documents came from.
    pub repository: String,
    /// The commit they were read at.
    pub commit: Commit,
    /// When this snapshot was taken, as an ISO-8601 instant. Distinct from the
    /// commit date: it says how stale the *check* is, not the content.
    pub fetched_at: String,
    /// Every study protocol at that commit.
    pub documents: Vec<ProtocolDocument>,
}

impl Snapshot {
    /// Build a snapshot from a completed fetch.
    pub fn new(repository: String, commit: Commit, documents: Vec<ProtocolDocument>) -> Self {
        Self {
            format_version: FORMAT_VERSION,
            repository,
            commit,
            fetched_at: chrono::Utc::now().to_rfc3339(),
            documents,
        }
    }

    /// Where a snapshot lives inside `data_dir`.
    pub fn path_in(data_dir: &Path) -> PathBuf {
        data_dir.join(FILE_NAME)
    }

    /// Read the snapshot from `data_dir`.
    ///
    /// Returns [`Error::Missing`] when nothing has been synced yet, and when
    /// what is stored was written by a build using a different layout.
    pub async fn load(data_dir: &Path) -> Result<Self> {
        let path = Self::path_in(data_dir);
        let contents = match tokio::fs::read_to_string(&path).await {
            Ok(contents) => contents,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(Error::Missing);
            }
            Err(source) => return Err(Error::Storage { path, source }),
        };

        let snapshot: Self = serde_json::from_str(&contents)?;
        if snapshot.format_version != FORMAT_VERSION {
            return Err(Error::Missing);
        }
        Ok(snapshot)
    }

    /// Write the snapshot into `data_dir`, replacing any earlier one.
    ///
    /// Written to a temporary file and renamed, so an interrupted write leaves
    /// the previous snapshot in place rather than a truncated one.
    pub async fn save(&self, data_dir: &Path) -> Result<()> {
        let path = Self::path_in(data_dir);
        let storage = |source| Error::Storage {
            path: path.clone(),
            source,
        };

        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(storage)?;
        }

        let contents = serde_json::to_vec(self)?;
        let temporary = path.with_extension("json.tmp");
        tokio::fs::write(&temporary, &contents)
            .await
            .map_err(storage)?;
        tokio::fs::rename(&temporary, &path)
            .await
            .map_err(storage)?;
        Ok(())
    }

    /// Names of the studies in the snapshot, in document order.
    pub fn study_names(&self) -> Vec<&str> {
        self.documents
            .iter()
            .map(|document| document.study.as_str())
            .collect()
    }

    /// One study's protocol, parsed.
    ///
    /// This is what "start from a template" uses, so it parses through
    /// [`carp_protocol`] rather than handing back raw JSON: a template that
    /// does not load is better discovered here than half way through an edit.
    pub fn template(&self, study: &str) -> Result<carp_protocol::StudyProtocol> {
        let document = self
            .documents
            .iter()
            .find(|document| document.study == study)
            .ok_or_else(|| {
                Error::Unexpected(format!("no study named {study:?} in the catalogue"))
            })?;
        Ok(serde_json::from_str(&document.json)?)
    }

    /// Every protocol in the snapshot that parses, with its study name.
    ///
    /// A document that fails to parse is skipped rather than fatal: one
    /// malformed study upstream must not make the whole catalogue unusable.
    /// [`crate::derive`] reports how many were skipped.
    pub fn parsed(&self) -> (Vec<(String, carp_protocol::StudyProtocol)>, Vec<String>) {
        let mut parsed = Vec::with_capacity(self.documents.len());
        let mut failed = Vec::new();

        for document in &self.documents {
            match serde_json::from_str(&document.json) {
                Ok(protocol) => parsed.push((document.study.clone(), protocol)),
                Err(error) => failed.push(format!("{}: {error}", document.study)),
            }
        }
        (parsed, failed)
    }
}

#[cfg(test)]
mod tests;
