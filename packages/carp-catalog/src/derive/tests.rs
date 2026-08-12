// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`].

use super::*;
use crate::catalog::CatalogEntry;
use crate::source::{Commit, ProtocolDocument};

/// A snapshot of the real reference protocols, so the derivation is
/// tested against what it will actually read rather than a mock.
fn snapshot() -> Snapshot {
    // Chosen to cover the range: a small phone study, a survey-heavy one,
    // a branching one, a browser-only one with no `applicationData`, the
    // app-store study - the only one reading the full health metric list -
    // and the CAMS 2.0 study, which is the only one using the newer
    // device namespace.
    let studies: [(&str, &str); 6] = [
        ("neuropathy", include_str!("../../../carp-protocol/tests/corpus/neuropathy.json")),
        ("demo", include_str!("../../../carp-protocol/tests/corpus/demo.json")),
        ("catch", include_str!("../../../carp-protocol/tests/corpus/catch.json")),
        ("icat", include_str!("../../../carp-protocol/tests/corpus/icat.json")),
        ("app_store", include_str!("../../../carp-protocol/tests/corpus/app_store.json")),
        ("test", include_str!("../../../carp-protocol/tests/corpus/test.json")),
    ];

    Snapshot::new(
        "carp-dk/carp_study_app_configurations".to_owned(),
        Commit {
            sha: "158cdcbe94980d48afc6478dd8f11c7ac4bbff5a".to_owned(),
            date: "2026-07-07T13:52:26+02:00".to_owned(),
            subject: "Update NT to protocol v2".to_owned(),
        },
        studies
            .into_iter()
            .map(|(study, json)| ProtocolDocument {
                study: study.to_owned(),
                path: format!("{study}/carp/resources/protocol.json"),
                json: json.to_owned(),
            })
            .collect(),
    )
}

fn values(vocabulary: &[CatalogEntry]) -> Vec<&str> {
    vocabulary.iter().map(|entry| entry.value.as_str()).collect()
}

/// The point of the crate: the vocabulary comes from the protocols, so
/// what the studies use is what the editor offers.
#[test]
fn the_vocabulary_comes_from_the_protocols() {
    let catalog = catalog(&snapshot());

    assert!(catalog.skipped.is_empty(), "{:?}", catalog.skipped);
    assert_eq!(catalog.version.as_ref().unwrap().studies, 6);

    let measures = values(&catalog.measure_types);
    assert!(measures.contains(&"dk.cachet.carp.survey"), "{measures:?}");
    assert!(measures.contains(&"dk.cachet.carp.location"), "{measures:?}");
    assert!(
        measures.contains(&"dk.cachet.carp.completedtask"),
        "{measures:?}"
    );

    let inputs = values(&catalog.input_data_types);
    assert!(
        inputs.contains(&"dk.carp.webservices.input.informed_consent"),
        "{inputs:?}"
    );

    let question_types = values(&catalog.question_types);
    assert!(question_types.contains(&"SingleChoice"), "{question_types:?}");

    assert!(values(&catalog.upload_methods).contains(&"stream"));
    assert!(values(&catalog.user_task_conditions).contains(&"done"));
}

/// Health metrics live in two places and both have to be found, or the
/// picker offers a fraction of what studies actually read.
#[test]
fn health_metrics_are_gathered_from_everywhere_they_appear() {
    let catalog = catalog(&snapshot());
    let metrics = values(&catalog.health_data_types);

    assert!(metrics.contains(&"STEPS"), "{metrics:?}");
    assert!(metrics.contains(&"SLEEP_SESSION"), "{metrics:?}");
    assert!(metrics.len() > 10, "expected the full metric list, got {metrics:?}");
}

/// A device class this build cannot construct must still reach the
/// catalogue, because the raw pass is what makes the tool honest about
/// what exists upstream.
#[test]
fn device_classes_are_read_from_the_raw_document() {
    let catalog = catalog(&snapshot());
    let devices = values(&catalog.device_types);

    assert!(
        devices.contains(&"dk.carp.cams.devices.Smartphone"),
        "the CAMS 2.0 namespace is in use upstream: {devices:?}"
    );
    assert!(
        devices.contains(&"dk.cachet.carp.common.application.devices.WebBrowser"),
        "{devices:?}"
    );
}

/// Entries have to carry which studies use them, since that is how the
/// editor orders and explains them.
#[test]
fn entries_know_which_studies_use_them() {
    let catalog = catalog(&snapshot());
    let survey = catalog
        .measure_types
        .iter()
        .find(|entry| entry.value == "dk.cachet.carp.survey")
        .expect("three of the four studies run surveys");

    assert!(survey.studies.contains(&"demo".to_owned()), "{survey:?}");
    assert!(!survey.studies.contains(&"icat".to_owned()), "ICAT is web-only");
    assert!(survey.occurrences >= survey.studies.len());
}

/// Every study becomes a template, described well enough to choose from.
#[test]
fn every_study_is_offered_as_a_template() {
    let catalog = catalog(&snapshot());
    assert_eq!(catalog.templates.len(), 6);

    let neuropathy = catalog
        .templates
        .iter()
        .find(|template| template.study == "neuropathy")
        .unwrap();
    assert_eq!(neuropathy.name, "CARP Neuropathy Tracker Protocol");
    assert_eq!(neuropathy.summary, "1 device, 3 tasks, 3 triggers");
    assert!(neuropathy.has_application_data);

    let icat = catalog
        .templates
        .iter()
        .find(|template| template.study == "icat")
        .unwrap();
    assert!(!icat.has_application_data, "ICAT targets the core runtime");
}

/// An unreadable document is reported rather than silently dropped.
#[test]
fn a_malformed_study_is_reported() {
    let mut snapshot = snapshot();
    snapshot.documents.push(ProtocolDocument {
        study: "broken".to_owned(),
        path: "broken/carp/resources/protocol.json".to_owned(),
        json: "{ not json".to_owned(),
    });

    let catalog = catalog(&snapshot);
    assert_eq!(catalog.skipped.len(), 1);
    assert_eq!(catalog.version.unwrap().studies, 6);
}

/// An empty snapshot must produce an empty catalogue, not a panic.
#[test]
fn an_empty_snapshot_produces_an_empty_catalogue() {
    let mut snapshot = snapshot();
    snapshot.documents.clear();

    let catalog = catalog(&snapshot);
    assert!(catalog.is_empty());
    assert_eq!(catalog.version.unwrap().studies, 0);
}
