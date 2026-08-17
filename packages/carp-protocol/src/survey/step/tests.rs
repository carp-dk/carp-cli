// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`].

use super::*;
use crate::survey::choice::RpChoice;

#[test]
fn an_instruction_step_round_trips() {
    let original = serde_json::json!({
        "__type": "RPInstructionStep",
        "identifier": "cvd_instruction",
        "title": "Cardiovascular Event",
        "text": "Report any event you experienced.",
        "optional": false
    });

    let step: RpStep = serde_json::from_value(original.clone()).unwrap();
    assert_eq!(step.identifier(), "cvd_instruction");
    assert_eq!(step.type_label(), "RPInstructionStep");
    assert_eq!(serde_json::to_value(&step).unwrap(), original);
}

#[test]
fn a_question_step_round_trips_with_its_format() {
    let original = serde_json::json!({
        "__type": "RPQuestionStep",
        "identifier": "survey.demographics.1",
        "title": "survey.demographics.question.sex",
        "optional": false,
        "answerFormat": {
            "__type": "RPChoiceAnswerFormat",
            "questionType": "SingleChoice",
            "choices": [
                {"__type": "RPChoice", "text": "f", "value": 1, "isFreeText": false}
            ],
            "answerStyle": "SingleChoice"
        },
        "autoSkip": false,
        "timeout": 0,
        "autoFocus": false
    });

    let step: RpStep = serde_json::from_value(original.clone()).unwrap();
    assert_eq!(
        step.answer_format().unwrap().label(),
        "SingleChoice, 1 options"
    );
    assert_eq!(serde_json::to_value(&step).unwrap(), original);
}

/// A form step nests questions, so the recursion has to hold.
#[test]
fn a_form_step_carries_its_questions() {
    let mut form: RpStep = serde_json::from_value(serde_json::json!({
        "__type": "RPFormStep",
        "identifier": "onboarding.smoking.form",
        "title": "onboarding.smoking.title",
        "optional": false,
        "answerFormat": { "__type": "RPFormAnswerFormat", "questionType": "Form" },
        "autoSkip": false,
        "timeout": 0,
        "autoFocus": false,
        "questions": []
    }))
    .unwrap();

    form.questions_mut().unwrap().push(RpStep::question(
        "q1",
        "How many?",
        RpAnswerFormat::single_choice(vec![RpChoice::new("One", 1)]),
    ));

    let json = serde_json::to_value(&form).unwrap();
    assert_eq!(json["questions"].as_array().unwrap().len(), 1);

    let parsed: RpStep = serde_json::from_value(json).unwrap();
    assert_eq!(parsed, form);
}

/// The optional flags of a form step must stay absent when they were.
#[test]
fn absent_form_flags_stay_absent() {
    let original = serde_json::json!({
        "__type": "RPFormStep",
        "identifier": "f",
        "title": "t",
        "optional": false,
        "answerFormat": { "__type": "RPFormAnswerFormat", "questionType": "Form" },
        "autoSkip": false,
        "timeout": 0,
        "autoFocus": false,
        "questions": []
    });

    let step: RpStep = serde_json::from_value(original.clone()).unwrap();
    let written = serde_json::to_value(&step).unwrap();
    assert!(written.get("forceWait").is_none(), "got {written}");
    assert!(
        written.get("saveResultsOnAutoSkip").is_none(),
        "got {written}"
    );
    assert_eq!(written, original);
}

#[test]
fn the_cognitive_activities_round_trip() {
    let activities = [
        serde_json::json!({
            "__type": "RPTappingActivity",
            "identifier": "tapping_1",
            "title": "RPActivityStep",
            "optional": false,
            "includeInstructions": true,
            "includeResults": true,
            "lengthOfTest": 10
        }),
        serde_json::json!({
            "__type": "RPFlankerActivity",
            "identifier": "flanker_1",
            "title": "RPActivityStep",
            "optional": false,
            "includeInstructions": true,
            "includeResults": true,
            "lengthOfTest": 30,
            "numberOfCards": 10
        }),
        serde_json::json!({
            "__type": "RPReactionTimeActivity",
            "identifier": "rt_1",
            "title": "RPActivityStep",
            "optional": false,
            "includeInstructions": true,
            "includeResults": true,
            "lengthOfTest": 24,
            "switchInterval": 4
        }),
    ];

    for original in activities {
        let step: RpStep = serde_json::from_value(original.clone()).unwrap();
        assert!(
            matches!(step, RpStep::Known(_)),
            "{original} fell through to the unknown fallback"
        );
        assert_eq!(serde_json::to_value(&step).unwrap(), original);
    }
}
