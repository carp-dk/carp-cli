// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Versioning a protocol.
//!
//! Three different versions travel with a protocol, and confusing them is easy:
//!
//! 1. **The revision** - [`StudyProtocol::version`], a counter starting at 0.
//!    CARP increments it each time a new version of the *same* protocol (same
//!    [`StudyProtocol::id`]) is stored.
//! 2. **The version tag** - a human label CAWS files that revision under, such
//!    as `v1.2.0`. `AddVersion` requires one, and CAWS rejects a tag already
//!    used for that protocol. See [`VersionTag`].
//! 3. **The protocol API level** - which release of the study app the document
//!    is written for, in `applicationData.protocolApiLevel`.
//!
//! A protocol is *never* edited in place once uploaded: a change means a new
//! revision under the same id, so deployments already running keep the
//! document they were created from. [`next_revision`] is what enforces that.

use serde::{Deserialize, Serialize};

use crate::protocol::StudyProtocol;
use crate::validate::{Severity, validate};

/// A version tag, as CAWS files revisions under.
///
/// Free-form on the wire, so this type does not constrain what can be read;
/// it exists to *produce* tags that sort and read sensibly, and to recognise
/// the `vMAJOR.MINOR.PATCH` shape well enough to offer the next one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VersionTag(pub String);

impl VersionTag {
    /// The tag a protocol's first revision gets.
    pub fn initial() -> Self {
        Self("v1.0.0".to_owned())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Read `v1.2.3` or `1.2.3` as its three numbers.
    ///
    /// Returns `None` for tags of any other shape, which are perfectly legal
    /// and simply cannot be incremented automatically.
    pub fn parts(&self) -> Option<(u32, u32, u32)> {
        let text = self.0.trim().trim_start_matches(['v', 'V']);
        let mut numbers = text.split('.');
        let major = numbers.next()?.parse().ok()?;
        let minor = numbers.next()?.parse().ok()?;
        let patch = numbers.next()?.parse().ok()?;
        numbers.next().is_none().then_some((major, minor, patch))
    }

    /// The next tag after this one, bumping `component`.
    ///
    /// A tag that is not `vMAJOR.MINOR.PATCH` cannot be incremented, so the
    /// caller is given `None` and should ask for a tag instead of inventing
    /// one - silently replacing someone's `pilot-2` with `v1.0.1` would lose
    /// information the tag was carrying.
    pub fn next(&self, component: Bump) -> Option<Self> {
        let (major, minor, patch) = self.parts()?;
        let (major, minor, patch) = match component {
            Bump::Major => (major + 1, 0, 0),
            Bump::Minor => (major, minor + 1, 0),
            Bump::Patch => (major, minor, patch + 1),
        };
        Some(Self(format!("v{major}.{minor}.{patch}")))
    }
}

impl std::fmt::Display for VersionTag {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Which part of a version tag to increment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bump {
    /// A change that breaks continuity with the data already collected: a
    /// measure removed, a survey's answers rescored.
    Major,
    /// Something added that does not invalidate what came before.
    Minor,
    /// A correction with no effect on the data: a typo, a translation key.
    Patch,
}

impl Bump {
    pub const ALL: [Self; 3] = [Self::Patch, Self::Minor, Self::Major];

    pub fn label(self) -> &'static str {
        match self {
            Self::Major => "major",
            Self::Minor => "minor",
            Self::Patch => "patch",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Major => "Breaks continuity with data already collected",
            Self::Minor => "Adds something without invalidating earlier data",
            Self::Patch => "A correction with no effect on the data",
        }
    }
}

/// One stored revision of a protocol, as CAWS reports it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolVersion {
    /// The version tag it was filed under.
    pub tag: String,
    /// When it was stored, as an ISO-8601 instant.
    pub date: String,
}

/// Prepare `protocol` as the next revision of itself.
///
/// Increments [`StudyProtocol::version`] and leaves the id alone, which is
/// what makes it a new *version* rather than a new *protocol*. Returns the
/// revision number it moved to.
pub fn next_revision(protocol: &mut StudyProtocol) -> u32 {
    protocol.version = protocol.version.saturating_add(1);
    protocol.version
}

/// Prepare `protocol` as a brand-new protocol derived from this one.
///
/// Gives it a fresh id and resets the revision to 0, so uploading it creates
/// a separate protocol rather than a version of the original. This is what
/// "duplicate" in the editor does, and the reason it is not just a clone:
/// keeping the id would make the copy overwrite the original's history.
pub fn fork(protocol: &mut StudyProtocol, name: impl Into<String>) {
    protocol.id = uuid::Uuid::new_v4().to_string();
    protocol.version = 0;
    protocol.name = name.into();
    protocol.created_on = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true);
}

/// Why a protocol cannot be uploaded to CAWS.
///
/// Stricter than [`crate::validate()`], which judges a protocol as a document. This
/// judges it as an upload payload, where CAWS' own requirements apply: it
/// rejects a non-UUID owner, and a protocol with no primary device produces a
/// study nothing can deploy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadCheck {
    /// Reasons the upload would be rejected. Empty means it is ready.
    pub blockers: Vec<String>,
}

impl UploadCheck {
    /// Judge `protocol` as an upload payload.
    pub fn run(protocol: &StudyProtocol) -> Self {
        let mut blockers: Vec<String> = validate(protocol)
            .into_iter()
            .filter(|diagnostic| diagnostic.severity == Severity::Error)
            .map(|diagnostic| format!("{}: {}", diagnostic.location, diagnostic.message))
            .collect();

        // CAWS stores the owner as a UUID column, so a placeholder that the
        // validator tolerates in a source document is fatal here.
        if uuid::Uuid::parse_str(&protocol.owner_id).is_err() {
            blockers.push(format!(
                "protocol: owner id {:?} is not a UUID, which CAWS requires",
                protocol.owner_id
            ));
        }

        Self { blockers }
    }

    pub fn is_ready(&self) -> bool {
        self.blockers.is_empty()
    }
}

#[cfg(test)]
mod tests;
