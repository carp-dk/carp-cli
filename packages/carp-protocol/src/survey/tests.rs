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
fn an_ordered_survey_round_trips() {
    let original = serde_json::json!({
        "__type": "RPOrderedTask",
        "identifier": "neuropathy_assessment",
        "closeAfterFinished": true,
        "steps": [{
            "__type": "RPInstructionStep",
            "identifier": "neuropathy_assessment_instruction",
            "title": "Neuropathy Task",
            "text": "General symptoms.",
            "optional": false
        }]
    });

    let task: RpTask = serde_json::from_value(original.clone()).unwrap();
    assert_eq!(task.identifier(), "neuropathy_assessment");
    assert_eq!(task.steps().len(), 1);
    assert_eq!(serde_json::to_value(&task).unwrap(), original);
}

#[test]
fn a_navigable_survey_round_trips_with_its_rules() {
    let original = serde_json::json!({
        "__type": "RPNavigableOrderedTask",
        "identifier": "OnBoardingSurvey",
        "closeAfterFinished": true,
        "steps": [],
        "stepNavigationRules": {
            "onboarding.smoking.step": {
                "__type": "RPStepJumpRule",
                "answerMap": { "1": "a.step", "0": "b.step" }
            }
        }
    });

    let task: RpTask = serde_json::from_value(original.clone()).unwrap();
    assert_eq!(task.navigation_rules().unwrap().len(), 1);
    assert_eq!(serde_json::to_value(&task).unwrap(), original);
}

/// Switching kinds must keep the steps: losing a survey's questions
/// because a checkbox was ticked would be unforgivable.
#[test]
fn switching_kind_keeps_the_steps() {
    let mut task = RpTask::ordered("s");
    task.steps_mut()
        .unwrap()
        .push(RpStep::instruction("i", "Title", "Text"));

    task.set_navigable(true);
    assert_eq!(task.type_label(), "RPNavigableOrderedTask");
    assert_eq!(task.steps().len(), 1);
    assert_eq!(task.identifier(), "s");

    task.set_navigable(false);
    assert_eq!(task.type_label(), "RPOrderedTask");
    assert_eq!(task.steps().len(), 1);
    assert_eq!(task.identifier(), "s");
}

/// Validation needs the identifiers of questions nested inside forms, not
/// just the pages at the top level.
#[test]
fn nested_questions_are_walked() {
    let mut form: RpStep = serde_json::from_value(serde_json::json!({
        "__type": "RPFormStep",
        "identifier": "form",
        "title": "t",
        "optional": false,
        "answerFormat": { "__type": "RPFormAnswerFormat", "questionType": "Form" },
        "autoSkip": false,
        "timeout": 0,
        "autoFocus": false,
        "questions": []
    }))
    .unwrap();
    form.questions_mut().unwrap().push(RpStep::question(
        "nested",
        "How many?",
        RpAnswerFormat::integer(0, 10, ""),
    ));

    let mut task = RpTask::ordered("s");
    let steps = task.steps_mut().unwrap();
    steps.push(RpStep::instruction("intro", "Title", "Text"));
    steps.push(form);

    assert_eq!(task.all_step_identifiers(), ["intro", "form", "nested"]);
}
