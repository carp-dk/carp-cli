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
fn an_upload_says_what_it_did() {
    let created = StoreOutcome::Stored {
        tag: "v1.0.0".to_owned(),
        first: true,
        revisions: 1,
    };
    assert_eq!(
        created.message(),
        "uploaded as a new protocol, tagged v1.0.0"
    );
    assert!(created.is_stored());

    let revised = StoreOutcome::Stored {
        tag: "v1.1.0".to_owned(),
        first: false,
        revisions: 3,
    };
    assert_eq!(
        revised.message(),
        "uploaded as v1.1.0 - CAWS now holds 3 revisions"
    );
}

/// A rejected tag has to say what to do, since the fix is to pick another.
#[test]
fn a_taken_tag_explains_itself() {
    let taken = StoreOutcome::TagTaken {
        tag: "v1.0.0".to_owned(),
        existing: 2,
    };
    assert!(taken.message().contains("choose another tag"), "{taken:?}");
    assert!(!taken.is_stored());
}
