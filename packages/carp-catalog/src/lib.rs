// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! What the editor knows about CARP, learned from the protocols CARP ships.
//!
//! [`carp_protocol`] models the *shape* of a protocol: that a measure has a
//! type, that a device has a role name. It cannot model the *vocabulary* -
//! which measure types exist, which health metrics can be read, which
//! localisation-key conventions a study follows - because none of that is
//! fixed. Every sampling package a study app links in contributes its own
//! measure types, and the set changes release to release.
//!
//! Hard-coding that vocabulary would mean the editor started going stale the
//! day it was written. So it is not hard-coded: it is *derived* from the
//! protocols in
//! [`carp_study_app_configurations`](https://github.com/carp-dk/carp_study_app_configurations),
//! the repository those studies are actually built from.
//!
//! # How a catalogue is produced
//!
//! 1. [`source`] asks the GitHub API for the current commit of the upstream
//!    repository, then for the `protocol.json` of every study in it.
//! 2. [`snapshot`] stores those documents on disk, tagged with the commit they
//!    came from.
//! 3. [`mod@derive`] reads the snapshot and counts what it finds: every measure
//!    type, every device class, every health metric, every question type, and
//!    which studies use each.
//! 4. [`catalog`] is the result - the lists the editor offers, each entry
//!    knowing how widely it is used upstream.
//!
//! # Versioning
//!
//! A [`Catalog`] always knows the commit it came from ([`CatalogVersion`]), so
//! the editor can say *which* version of CARP's conventions it is offering,
//! and [`sync::check_for_updates`] can tell when upstream has moved on. A
//! catalogue is never silently replaced: syncing is something the user asks
//! for, and the version they were working against is recorded.
//!
//! The snapshot format has its own version, [`snapshot::FORMAT_VERSION`], so a
//! snapshot written by an older build is re-derived rather than misread.

pub mod catalog;
pub mod derive;
pub mod snapshot;
pub mod source;
pub mod sync;

pub use catalog::{Catalog, CatalogEntry, CatalogVersion};
pub use snapshot::Snapshot;
pub use source::{Commit, GitHubSource, ProtocolDocument};
pub use sync::{SyncOutcome, SyncReport};

/// Owner and name of the repository the vocabulary is derived from.
pub const UPSTREAM_OWNER: &str = "carp-dk";
pub const UPSTREAM_REPO: &str = "carp_study_app_configurations";
/// Branch followed when no other reference is given.
pub const UPSTREAM_BRANCH: &str = "main";

/// Anything that can go wrong fetching or reading a catalogue.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The GitHub API could not be reached, or refused the request.
    #[error("could not reach the {UPSTREAM_OWNER}/{UPSTREAM_REPO} repository: {0}")]
    Upstream(#[from] reqwest::Error),

    /// GitHub answered, but not with what was asked for.
    #[error("unexpected answer from GitHub: {0}")]
    Unexpected(String),

    /// A snapshot on disk could not be read or written.
    #[error("cannot use the catalogue at {path}: {source}")]
    Storage {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// A stored snapshot is not in a shape this build understands.
    #[error("the stored catalogue is unreadable: {0}")]
    Corrupt(#[from] serde_json::Error),

    /// No catalogue has been synced yet.
    #[error("no catalogue has been downloaded yet; run `carp protocol sync`")]
    Missing,
}

/// Result alias used throughout the crate.
pub type Result<T> = std::result::Result<T, Error>;
