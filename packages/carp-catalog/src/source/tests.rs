// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`].

use super::*;

#[test]
fn a_study_is_named_by_its_directory() {
    assert_eq!(
        study_name("neuropathy/carp/resources/protocol.json"),
        "neuropathy"
    );
    assert_eq!(study_name("protocol.json"), "protocol.json");
}

/// The short SHA is what the interface shows, and must not panic on a
/// value shorter than seven characters.
#[test]
fn short_shas_are_seven_characters() {
    let commit = |sha: &str| Commit {
        sha: sha.to_owned(),
        date: String::new(),
        subject: String::new(),
    };
    assert_eq!(
        commit("158cdcbe94980d48afc6478dd8f11c7ac4bbff5a").short_sha(),
        "158cdcb"
    );
    assert_eq!(commit("abc").short_sha(), "abc");
    assert_eq!(commit("").short_sha(), "");
}

/// The path filter is what decides which files a sync costs a request
/// for, so it must match the layout and nothing else.
#[test]
fn only_study_protocols_match_the_path_filter() {
    let matches = |path: &str| path.ends_with(PROTOCOL_SUFFIX);

    assert!(matches("neuropathy/carp/resources/protocol.json"));
    assert!(matches("app_store/carp/resources/protocol.json"));
    assert!(!matches("neuropathy/carp/lang/en.json"));
    assert!(!matches("neuropathy/pubspec.yaml"));
    assert!(!matches("templates/protocol.json"));
}
