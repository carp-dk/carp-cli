// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`].

use super::*;

/// The common choice has four fields; a detail line must not appear as
/// `null` when it is absent.
#[test]
fn a_plain_choice_omits_its_detail_line() {
    let choice = RpChoice::new("survey.demographics.male", 2);
    assert_eq!(
        serde_json::to_value(&choice).unwrap(),
        serde_json::json!({
            "__type": "RPChoice",
            "text": "survey.demographics.male",
            "value": 2,
            "isFreeText": false
        })
    );
}

#[test]
fn a_detailed_choice_round_trips() {
    let original = serde_json::json!({
        "__type": "RPChoice",
        "text": "Other",
        "value": 9,
        "detailText": "Please describe below",
        "isFreeText": true
    });

    let choice: RpChoice = serde_json::from_value(original.clone()).unwrap();
    assert!(choice.is_free_text);
    assert_eq!(choice.integer_value(), Some(9));
    assert_eq!(serde_json::to_value(&choice).unwrap(), original);
}

/// A value need not be an integer, and a non-integer one must survive.
#[test]
fn a_non_integer_value_is_preserved() {
    let original = serde_json::json!({
        "__type": "RPChoice",
        "text": "Prefer not to say",
        "value": "unknown",
        "isFreeText": false
    });

    let choice: RpChoice = serde_json::from_value(original.clone()).unwrap();
    assert_eq!(choice.integer_value(), None);
    assert_eq!(serde_json::to_value(&choice).unwrap(), original);
}

#[test]
fn an_image_choice_round_trips() {
    let original = serde_json::json!({
        "__type": "RPImageChoice",
        "imageUrl": "assets/icons/very-sad.png",
        "value": 1,
        "description": "survey.sleep.image1"
    });

    let choice: RpImageChoice = serde_json::from_value(original.clone()).unwrap();
    assert_eq!(serde_json::to_value(&choice).unwrap(), original);
}
