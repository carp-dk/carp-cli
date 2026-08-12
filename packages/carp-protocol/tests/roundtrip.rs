// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! The model checked against every protocol CARP actually ships.
//!
//! `tests/corpus/` holds the `protocol.json` of each study in
//! [`carp_study_app_configurations`], vendored at the commit recorded in
//! `tests/corpus/SOURCE.txt`. Those files are the specification this crate is
//! written against, so they are the specification it is tested against:
//!
//! - [`every_protocol_round_trips`] proves nothing is lost or invented. It is
//!   the test that matters: if the model drops a field, mis-spells a key or
//!   changes a number's type, the re-serialised document stops matching.
//! - [`no_protocol_falls_back_to_unknown_nodes`] proves the round trip is
//!   honest. The [`carp_protocol::node`] fallback preserves anything it does
//!   not understand, which would let a modelling bug pass the first test by
//!   quietly turning a device into an opaque blob.
//!
//! [`carp_study_app_configurations`]: https://github.com/carp-dk/carp_study_app_configurations

use std::path::{Path, PathBuf};

use carp_protocol::StudyProtocol;
use carp_protocol::validate::Severity;
use serde_json::Value;

mod support;
use support::{difference, unmodelled_types};

/// Directory holding the vendored reference protocols.
fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus")
}

/// Every `<study>.json` in the corpus, as (name, contents).
fn corpus() -> Vec<(String, String)> {
    let mut protocols: Vec<(String, String)> = std::fs::read_dir(corpus_dir())
        .expect("the corpus directory is part of the crate")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .map(|path| {
            let name = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            let contents = std::fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("reading {}: {error}", path.display()));
            (name, contents)
        })
        .collect();
    protocols.sort_by(|left, right| left.0.cmp(&right.0));

    assert!(
        protocols.len() >= 10,
        "expected the full corpus, found {} file(s)",
        protocols.len()
    );
    protocols
}

/// Parsing and re-serialising a protocol must reproduce it exactly.
///
/// The comparison is on `serde_json::Value`, so key order does not matter but
/// every key, value and JSON type does. A dropped field, a `10.0` written back
/// as `10`, or a `null` where the original had nothing all fail here.
#[test]
fn every_protocol_round_trips() {
    for (name, json) in corpus() {
        let original: Value = serde_json::from_str(&json)
            .unwrap_or_else(|error| panic!("{name}: the fixture is not valid JSON: {error}"));

        let protocol: StudyProtocol = serde_json::from_str(&json)
            .unwrap_or_else(|error| panic!("{name}: did not parse as a protocol: {error}"));

        let written = serde_json::to_value(&protocol)
            .unwrap_or_else(|error| panic!("{name}: did not serialise: {error}"));

        if written != original {
            panic!(
                "{name} changed on a round trip\n{}",
                difference(&original, &written)
            );
        }
    }
}

/// No reference protocol may rely on the unknown-node fallback.
///
/// The fallback exists for protocols from a future CARP release. If one of
/// today's uses it, this crate has failed to model something it should - and
/// [`every_protocol_round_trips`] would not notice, because the fallback
/// round-trips perfectly by design.
#[test]
fn no_protocol_falls_back_to_unknown_nodes() {
    for (name, json) in corpus() {
        let protocol: StudyProtocol = serde_json::from_str(&json).expect("parses");

        // A modelled value serialises from a typed variant; an unknown one is
        // reproduced from its raw fields. Re-reading the written document as
        // a `Value` and comparing the set of `__type` strings against the ones
        // the model recognises finds any that only survived verbatim.
        let unmodelled = unmodelled_types(&serde_json::to_value(&protocol).unwrap());
        assert!(
            unmodelled.is_empty(),
            "{name} contains types this crate does not model: {unmodelled:?}\n\
             Model them in carp-protocol rather than leaving them to the fallback."
        );
    }
}

/// Faults the validator finds in the upstream protocols themselves.
///
/// These are not shortcomings of the validator - they are real defects in
/// documents currently in production, and the rules that find them are
/// correct. They are listed so [`every_protocol_validates_without_errors`] can
/// hold every *other* protocol to a clean bill of health while recording what
/// is genuinely wrong upstream.
///
/// Each entry is `(study, location, message)` and should be removed once the
/// corresponding fix lands in `carp_study_app_configurations`.
const KNOWN_UPSTREAM_DEFECTS: &[(&str, &str, &str)] = &[
    // The trigger was given the localisation key of the task's title instead
    // of the task's name, so it watches a task that does not exist. Trigger 15
    // is a `NoUserTaskTrigger` meant to re-offer "Report Symptoms".
    (
        "catch",
        "trigger 15",
        "watches \"pro.instruction.title\", which is not a task in this protocol",
    ),
];

/// Every reference protocol must pass validation with no errors, bar the
/// upstream defects recorded in [`KNOWN_UPSTREAM_DEFECTS`].
///
/// These are the protocols in production, so anything else this crate calls an
/// error would be a false positive - and false positives train users to ignore
/// the validator, which is worse than not having one.
#[test]
fn every_protocol_validates_without_errors() {
    let mut unexpected = Vec::new();
    let mut matched = Vec::new();

    for (name, json) in corpus() {
        let protocol: StudyProtocol = serde_json::from_str(&json).expect("parses");

        for diagnostic in carp_protocol::validate(&protocol) {
            if diagnostic.severity != Severity::Error {
                continue;
            }
            let entry = (
                name.as_str(),
                diagnostic.location.as_str(),
                diagnostic.message.as_str(),
            );
            if KNOWN_UPSTREAM_DEFECTS.contains(&entry) {
                matched.push((name.clone(), diagnostic.location.clone()));
            } else {
                unexpected.push(format!(
                    "  {name}: [{}] {}",
                    diagnostic.location, diagnostic.message
                ));
            }
        }
    }

    assert!(
        unexpected.is_empty(),
        "unexpected validation errors in the reference protocols:\n{}",
        unexpected.join("\n")
    );

    // A defect that has been fixed upstream must be taken off the list, or it
    // silently stops being checked.
    assert_eq!(
        matched.len(),
        KNOWN_UPSTREAM_DEFECTS.len(),
        "a recorded upstream defect no longer occurs; remove it from \
         KNOWN_UPSTREAM_DEFECTS. Still found: {matched:?}"
    );
}

/// The pretty printer must produce a document that parses back identically,
/// since that is what the editor writes to disk.
#[test]
fn the_pretty_printer_produces_readable_json_that_parses_back() {
    for (name, json) in corpus() {
        let protocol: StudyProtocol = serde_json::from_str(&json).expect("parses");
        let pretty = carp_protocol::to_json(&protocol).expect("serialises");

        assert!(
            pretty.contains("\n  \""),
            "{name}: expected two-space indented output"
        );

        let reparsed: Value = serde_json::from_str(&pretty).expect("pretty output is valid JSON");
        assert_eq!(
            reparsed,
            serde_json::from_str::<Value>(&json).unwrap(),
            "{name}: pretty printing changed the document"
        );
    }
}
