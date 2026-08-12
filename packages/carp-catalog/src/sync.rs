// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Keeping the catalogue current, and knowing when it is not.
//!
//! Two operations, deliberately separate:
//!
//! - [`check_for_updates`] costs one request and changes nothing. It answers
//!   "has upstream moved since the catalogue was taken?".
//! - [`sync`] downloads and replaces the catalogue.
//!
//! They are separate because replacing the catalogue changes what the editor
//! offers, and doing that behind someone's back mid-edit is unhelpful: a value
//! that was in the list a moment ago should not vanish. So the check runs
//! quietly, the result is shown, and the sync happens when it is asked for.
//!
//! [`load_or_sync`] is the exception, used at first run: with no catalogue at
//! all there is nothing to disturb.

use std::path::Path;

use crate::catalog::Catalog;
use crate::snapshot::Snapshot;
use crate::source::{Commit, GitHubSource};
use crate::{Error, Result, UPSTREAM_BRANCH, UPSTREAM_OWNER, UPSTREAM_REPO};

/// What a sync did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncOutcome {
    /// There was no catalogue; one was downloaded.
    Created,
    /// Upstream had moved; the catalogue was replaced.
    Updated,
    /// The catalogue was already at the current commit.
    AlreadyCurrent,
}

impl SyncOutcome {
    pub fn label(self) -> &'static str {
        match self {
            Self::Created => "catalogue downloaded",
            Self::Updated => "catalogue updated",
            Self::AlreadyCurrent => "catalogue already current",
        }
    }
}

/// The result of a sync: what happened, and what the catalogue now holds.
#[derive(Debug, Clone)]
pub struct SyncReport {
    pub outcome: SyncOutcome,
    /// The commit the catalogue is now at.
    pub commit: Commit,
    /// The commit it was at before, when there was one.
    pub previous: Option<Commit>,
    /// The catalogue derived from the new snapshot.
    pub catalog: Catalog,
}

impl SyncReport {
    /// A sentence for the status line.
    pub fn summary(&self) -> String {
        let studies = self.catalog.version.as_ref().map_or(0, |version| version.studies);
        match self.outcome {
            SyncOutcome::AlreadyCurrent => format!(
                "{} at {} ({studies} studies)",
                self.outcome.label(),
                self.commit.short_sha()
            ),
            _ => match &self.previous {
                Some(previous) => format!(
                    "{}: {} → {} ({studies} studies)",
                    self.outcome.label(),
                    previous.short_sha(),
                    self.commit.short_sha()
                ),
                None => format!(
                    "{} at {} ({studies} studies)",
                    self.outcome.label(),
                    self.commit.short_sha()
                ),
            },
        }
    }
}

/// Whether upstream has moved since the stored catalogue was taken.
///
/// Costs one API request and writes nothing. `Ok(None)` means the catalogue is
/// current; `Ok(Some(commit))` names the commit it could be updated to.
///
/// A missing catalogue is *not* an error here: at first run there is nothing
/// to compare against, and the answer is simply that a sync is available.
pub async fn check_for_updates(data_dir: &Path) -> Result<Option<Commit>> {
    let source = source()?;
    let head = source.head(UPSTREAM_BRANCH).await?;

    match Snapshot::load(data_dir).await {
        Ok(snapshot) if snapshot.commit.sha == head.sha => Ok(None),
        Ok(_) | Err(Error::Missing) => Ok(Some(head)),
        Err(error) => Err(error),
    }
}

/// Download the current upstream protocols and replace the catalogue.
///
/// Nothing is written until every document has been fetched, so a download
/// that fails part way leaves the previous catalogue intact rather than a
/// half-updated one.
pub async fn sync(data_dir: &Path) -> Result<SyncReport> {
    sync_reference(data_dir, UPSTREAM_BRANCH).await
}

/// Sync to a specific branch, tag or commit.
///
/// Pinning to a tag is how a study group holds the whole team to one version
/// of the conventions while a protocol is being written.
pub async fn sync_reference(data_dir: &Path, reference: &str) -> Result<SyncReport> {
    let source = source()?;
    let head = source.head(reference).await?;
    let previous = Snapshot::load(data_dir).await.ok().map(|snapshot| snapshot.commit);

    // Nothing to do, but the catalogue is still derived and returned so the
    // caller has one either way.
    if previous.as_ref().is_some_and(|commit| commit.sha == head.sha) {
        let snapshot = Snapshot::load(data_dir).await?;
        return Ok(SyncReport {
            outcome: SyncOutcome::AlreadyCurrent,
            commit: head,
            previous,
            catalog: crate::derive::catalog(&snapshot),
        });
    }

    let documents = source.documents(&head.sha).await?;
    let snapshot = Snapshot::new(
        format!("{UPSTREAM_OWNER}/{UPSTREAM_REPO}"),
        head.clone(),
        documents,
    );
    snapshot.save(data_dir).await?;

    Ok(SyncReport {
        outcome: if previous.is_some() {
            SyncOutcome::Updated
        } else {
            SyncOutcome::Created
        },
        commit: head,
        previous,
        catalog: crate::derive::catalog(&snapshot),
    })
}

/// The stored catalogue, without touching the network.
///
/// Returns [`Error::Missing`] when nothing has been synced. This is what the
/// editor calls at startup: it must not block on a network request, and
/// working offline against yesterday's catalogue is entirely reasonable.
pub async fn load(data_dir: &Path) -> Result<Catalog> {
    let snapshot = Snapshot::load(data_dir).await?;
    Ok(crate::derive::catalog(&snapshot))
}

/// The stored catalogue, downloading one if there is none.
///
/// For first run only. When a catalogue already exists it is returned as is,
/// however old - deciding to replace it is the user's.
pub async fn load_or_sync(data_dir: &Path) -> Result<Catalog> {
    match load(data_dir).await {
        Ok(catalog) => Ok(catalog),
        Err(Error::Missing) => Ok(sync(data_dir).await?.catalog),
        Err(error) => Err(error),
    }
}

/// The stored snapshot, for the templates it holds.
pub async fn load_snapshot(data_dir: &Path) -> Result<Snapshot> {
    Snapshot::load(data_dir).await
}

/// A source, authenticated from `GITHUB_TOKEN` when it is set.
///
/// Unauthenticated GitHub allows 60 requests an hour per address, which one
/// sync fits inside comfortably. Behind shared egress it may not, and a token
/// is the documented remedy.
fn source() -> Result<GitHubSource> {
    let source = GitHubSource::new()?;
    Ok(match std::env::var("GITHUB_TOKEN") {
        Ok(token) if !token.trim().is_empty() => source.with_token(token),
        _ => source,
    })
}

#[cfg(test)]
mod tests;
