// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`].

use super::*;

/// `questionType` and `answerStyle` say the same thing twice and CAMS
/// reads the second, so a builder that set only the first would render a
/// single-choice question as multiple choice.
#[test]
fn a_choice_format_states_its_style_consistently() {
    for (build, expected) in [
        (
            RpAnswerFormat::single_choice(vec![RpChoice::new("Yes", 1)]),
            "SingleChoice",
        ),
        (
            RpAnswerFormat::multiple_choice(vec![RpChoice::new("Yes", 1)]),
            "MultipleChoice",
        ),
    ] {
        let json = serde_json::to_value(&build).unwrap();
        assert_eq!(json["questionType"], expected);
        assert_eq!(json["answerStyle"], expected);
    }
}

/// The same doubling exists on date/time formats.
#[test]
fn a_date_time_format_states_its_style_consistently() {
    let json = serde_json::to_value(RpAnswerFormat::date_time("TimeOfDay")).unwrap();
    assert_eq!(json["questionType"], "TimeOfDay");
    assert_eq!(json["dateTimeAnswerStyle"], "TimeOfDay");
}

#[test]
fn every_modelled_format_round_trips() {
    let formats = [
        serde_json::json!({
            "__type": "RPChoiceAnswerFormat",
            "questionType": "SingleChoice",
            "choices": [{"__type": "RPChoice", "text": "a", "value": 1, "isFreeText": false}],
            "answerStyle": "SingleChoice"
        }),
        serde_json::json!({
            "__type": "RPImageChoiceAnswerFormat",
            "choices": [{
                "__type": "RPImageChoice",
                "imageUrl": "assets/a.png",
                "value": 1,
                "description": "a"
            }],
            "questionType": "ImageChoice"
        }),
        serde_json::json!({
            "__type": "RPIntegerAnswerFormat",
            "minValue": 0,
            "maxValue": 20,
            "suffix": "minutes",
            "questionType": "Integer"
        }),
        serde_json::json!({
            "__type": "RPSliderAnswerFormat",
            "minValue": 0.0,
            "maxValue": 10.0,
            "divisions": 10,
            "prefix": "",
            "suffix": "",
            "questionType": "Scale"
        }),
        serde_json::json!({
            "__type": "RPTextAnswerFormat",
            "hintText": "Add note...",
            "autoFocus": false,
            "disableHelpers": false,
            "questionType": "Text"
        }),
        serde_json::json!({
            "__type": "RPDateTimeAnswerFormat",
            "questionType": "DateAndTime",
            "dateTimeAnswerStyle": "DateAndTime"
        }),
        serde_json::json!({ "__type": "RPFormAnswerFormat", "questionType": "Form" }),
    ];

    for original in formats {
        let format: RpAnswerFormat = serde_json::from_value(original.clone()).unwrap();
        assert!(
            matches!(format, RpAnswerFormat::Known(_)),
            "{original} fell through to the unknown fallback"
        );
        assert_eq!(serde_json::to_value(&format).unwrap(), original);
    }
}

/// A text format without a hint must not gain a null one.
#[test]
fn an_absent_hint_stays_absent() {
    let json = serde_json::to_value(RpAnswerFormat::text(None)).unwrap();
    assert!(json.get("hintText").is_none(), "got {json}");
}
