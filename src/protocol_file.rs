// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Reading and writing `protocol.json`.
//!
//! The file this editor produces is meant to sit where the Flutter generator
//! used to put it - `<study>/carp/resources/protocol.json` in
//! `carp_study_app_configurations` - and to give a readable diff against the
//! previous one. So it is written pretty-printed with the same two-space
//! indentation, and the field order of [`carp_protocol`] matches what the Dart
//! generator emitted.
//!
//! Writes go through a temporary file and a rename. A protocol is often the
//! only record of an afternoon's work, and a half-written one would be worse
//! than no write at all.
//!
//! Reading is what `carp protocol check` and `show` need; writing is the
//! editor's. A build without the `tui` feature therefore uses only half of
//! this, which is a fact about that build rather than a reason to drop the
//! other half - the tests exercise both either way.

#![cfg_attr(
    not(feature = "tui"),
    allow(dead_code, reason = "the write half belongs to the editor")
)]

use std::path::{Path, PathBuf};

use carp_protocol::StudyProtocol;
use color_eyre::Result;
use color_eyre::eyre::{Context, bail};

/// Where a protocol lives inside a study directory, matching the layout of
/// `carp_study_app_configurations`.
pub const STUDY_RELATIVE_PATH: &str = "carp/resources/protocol.json";

/// Read a protocol from `path`.
///
/// Accepts either the file itself or the study directory containing it, since
/// both are things someone would reasonably point at.
pub fn read(path: &Path) -> Result<(StudyProtocol, PathBuf)> {
    let path = resolve(path);
    let json =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let protocol = carp_protocol::parse(&json)
        .with_context(|| format!("{} is not a CARP protocol", path.display()))?;
    Ok((protocol, path))
}

/// Write `protocol` to `path`, creating the directory if needed.
pub fn write(protocol: &StudyProtocol, path: &Path) -> Result<()> {
    let json = carp_protocol::to_json(protocol).context("rendering the protocol")?;

    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }

    // Written beside the target so the rename stays on one filesystem, which
    // is what makes it atomic.
    let temporary = path.with_extension("json.tmp");
    std::fs::write(&temporary, json.as_bytes())
        .with_context(|| format!("writing {}", temporary.display()))?;
    std::fs::rename(&temporary, path).with_context(|| format!("replacing {}", path.display()))?;
    Ok(())
}

/// The file a path refers to.
///
/// A directory is taken as a study directory, so pointing at `neuropathy/`
/// finds `neuropathy/carp/resources/protocol.json`. A path with no extension
/// that does not exist is taken as a study directory too, which is what makes
/// `carp protocol new sleep-study` do the expected thing.
pub fn resolve(path: &Path) -> PathBuf {
    if path.is_dir() {
        return path.join(STUDY_RELATIVE_PATH);
    }
    if path.extension().is_none() {
        return path.join(STUDY_RELATIVE_PATH);
    }
    path.to_path_buf()
}

/// Where a protocol should be saved when it has no path yet.
///
/// Derived from the protocol's own name so that saving does not have to ask,
/// and placed under `download_dir` because that is the directory the CLI
/// already owns and the user already knows about.
pub fn default_path(protocol: &StudyProtocol, base: &Path) -> PathBuf {
    base.join("protocols")
        .join(slug(&protocol.name))
        .join(STUDY_RELATIVE_PATH)
}

/// A file-name-safe form of `name`: lower case, words joined by hyphens.
fn slug(name: &str) -> String {
    let mut slug = String::with_capacity(name.len());
    let mut pending = false;

    for character in name.chars() {
        if character.is_alphanumeric() {
            if pending && !slug.is_empty() {
                slug.push('-');
            }
            pending = false;
            slug.extend(character.to_lowercase());
        } else {
            pending = true;
        }
    }

    if slug.is_empty() {
        "protocol".to_owned()
    } else {
        slug
    }
}

/// Read a protocol, failing with a clear message when the path is a
/// directory holding no protocol.
pub fn read_checked(path: &Path) -> Result<(StudyProtocol, PathBuf)> {
    let resolved = resolve(path);
    if !resolved.exists() {
        bail!(
            "no protocol at {}\n\
             Point at a protocol.json, or at a study directory containing {}",
            resolved.display(),
            STUDY_RELATIVE_PATH
        );
    }
    read(&resolved)
}

#[cfg(test)]
mod tests;
