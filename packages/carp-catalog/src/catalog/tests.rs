// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`].

use super::*;

#[test]
fn a_vocabulary_is_ordered_by_use_then_name() {
    let mut builder = VocabularyBuilder::default();
    builder.record("rare", "demo");
    for study in ["demo", "catch", "ubilife"] {
        builder.record("common", study);
    }
    builder.record("also-rare", "catch");

    let vocabulary = builder.build();
    let values: Vec<&str> = vocabulary.iter().map(|entry| entry.value.as_str()).collect();
    assert_eq!(values, ["common", "also-rare", "rare"]);
    assert_eq!(vocabulary[0].occurrences, 3);
    assert_eq!(vocabulary[0].studies, ["catch", "demo", "ubilife"]);
}

/// A study using a value twice counts twice, but is listed once.
#[test]
fn repeated_use_within_a_study_counts_once_in_the_study_list() {
    let mut builder = VocabularyBuilder::default();
    builder.record("value", "demo");
    builder.record("value", "demo");

    let vocabulary = builder.build();
    assert_eq!(vocabulary[0].occurrences, 2);
    assert_eq!(vocabulary[0].studies, ["demo"]);
}

#[test]
fn empty_values_are_not_recorded() {
    let mut builder = VocabularyBuilder::default();
    builder.record("", "demo");
    builder.record("   ", "demo");
    assert!(builder.build().is_empty());
}

/// Searching must find a type by its readable tail, since nobody types
/// the `dk.cachet.carp.` prefix.
#[test]
fn searching_matches_the_readable_tail() {
    let mut builder = VocabularyBuilder::default();
    builder.record("dk.cachet.carp.stepcount", "demo");
    builder.record("dk.cachet.carp.location", "demo");
    let vocabulary = builder.build();

    let found = Catalog::search(&vocabulary, "stepcount");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].short_value(), "stepcount");

    assert_eq!(Catalog::search(&vocabulary, "").len(), 2);
    assert!(Catalog::search(&vocabulary, "nothing").is_empty());
}

#[test]
fn usage_reads_as_a_phrase() {
    let entry = |studies: Vec<&str>| CatalogEntry {
        value: "v".to_owned(),
        occurrences: studies.len(),
        studies: studies.into_iter().map(str::to_owned).collect(),
    };
    assert_eq!(entry(vec![]).usage(), "unused");
    assert_eq!(entry(vec!["demo"]).usage(), "used by demo");
    assert_eq!(entry(vec!["demo", "catch"]).usage(), "used by 2 studies");
}

#[test]
fn a_version_reads_as_one_line() {
    let version = CatalogVersion {
        repository: "carp-dk/carp_study_app_configurations".to_owned(),
        commit: Commit {
            sha: "158cdcbe94980d48afc6478dd8f11c7ac4bbff5a".to_owned(),
            date: "2026-07-07T13:52:26+02:00".to_owned(),
            subject: "Update NT to protocol v2".to_owned(),
        },
        fetched_at: chrono::Utc::now().to_rfc3339(),
        studies: 10,
    };
    assert_eq!(version.label(), "158cdcb · 2026-07-07 · 10 studies");
    assert_eq!(version.age_in_days(), Some(0));
}
