// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests that renaming and deleting through a form keep every reference
//! intact, and that a refused value says why.
//!
//! Split from [`super::tests`], which owns the fixture and the round-trip
//! property, only so neither file grows past a screenful.

use super::tests::*;
use super::*;
use crate::app::form::{Subject, build};
use carp_protocol::builder;
use carp_protocol::trigger::TriggerKind;
use carp_protocol::validate::Severity;

/// Renaming through a form has to move every reference, exactly as renaming
/// through the builder does.
#[test]
fn renaming_a_device_through_its_form_moves_every_reference() {
    let mut protocol = protocol();
    let mut form = build::device(protocol.device("Primary Phone").unwrap());
    form.set_selected("Study Phone".to_owned());

    assert_eq!(apply(&mut protocol, &form), Applied::Changed);

    assert!(protocol.device("Study Phone").is_some());
    for trigger in protocol.triggers.values() {
        assert_ne!(trigger.source_device(), "Primary Phone");
    }
    assert!(errors(&protocol).is_empty(), "{:?}", errors(&protocol));
}

#[test]
fn renaming_a_task_through_its_form_moves_every_reference() {
    let mut protocol = protocol();
    let mut form = build::task(protocol.task("Survey").unwrap());
    form.set_selected("Sleep Diary".to_owned());

    assert_eq!(apply(&mut protocol, &form), Applied::Changed);

    assert!(protocol.task("Sleep Diary").is_some());
    assert!(
        protocol
            .triggers
            .values()
            .any(|trigger| trigger.watched_task() == Some("Sleep Diary")),
        "the watching trigger followed the rename"
    );
    assert!(errors(&protocol).is_empty(), "{:?}", errors(&protocol));
}

/// A rename onto a name already in use must be refused with a reason, not
/// performed and not silently dropped.
#[test]
fn a_colliding_rename_is_refused_with_a_reason() {
    let mut protocol = protocol();

    let mut form = build::device(protocol.device("Primary Phone").unwrap());
    form.set_selected("Location Service".to_owned());
    let outcome = apply(&mut protocol, &form);
    assert!(
        matches!(&outcome, Applied::Refused(reason) if reason.contains("already called")),
        "{outcome:?}"
    );
    assert!(protocol.device("Primary Phone").is_some());

    let mut form = build::task(protocol.task("Survey").unwrap());
    form.set_selected("Health Task".to_owned());
    assert!(matches!(apply(&mut protocol, &form), Applied::Refused(_)));
    assert!(protocol.task("Survey").is_some());
}

/// Changing a recurrence must move the period the phone schedules on, or the
/// editor and the phone disagree about when the survey arrives.
#[test]
fn changing_a_recurrence_moves_the_period_with_it() {
    let mut protocol = protocol();
    let (id, trigger) = protocol
        .triggers
        .iter()
        .find(|(_, trigger)| trigger.kind() == Some(TriggerKind::RecurrentScheduled))
        .map(|(id, trigger)| (*id, trigger.clone()))
        .unwrap();

    let mut form = build::trigger(id, &trigger, &protocol);
    let recurrence = form
        .fields
        .iter()
        .position(|field| field.key == "recurrence")
        .unwrap();
    form.selected = recurrence;
    form.set_selected("weekly".to_owned());

    assert_eq!(apply(&mut protocol, &form), Applied::Changed);

    let written = serde_json::to_value(&protocol.triggers[&id]).unwrap();
    assert_eq!(written["type"], "weekly");
    assert_eq!(written["period"], 604_800_000_000i64);
    assert_eq!(written["dayOfWeek"], 1);
}

/// A survey step's identifier is what branches point at, so renaming it has
/// to rewire them.
#[test]
fn renaming_a_survey_step_rewires_its_branches() {
    let mut protocol = protocol();

    // Make the survey branching, and add a rule pointing at the step.
    let survey = protocol
        .tasks
        .iter_mut()
        .find(|task| task.name() == "Survey")
        .and_then(carp_protocol::task::Task::survey_mut)
        .unwrap();
    survey.set_navigable(true);
    survey
        .steps_mut()
        .unwrap()
        .push(carp_protocol::survey::RpStep::instruction(
            "outro",
            "Thanks",
            "All done.",
        ));
    survey.navigation_rules_mut().unwrap().insert(
        "intro".to_owned(),
        carp_protocol::survey::RpStepNavigationRule::jump(std::collections::BTreeMap::from([(
            "0".to_owned(),
            "outro".to_owned(),
        )])),
    );

    let step = protocol.task("Survey").unwrap().survey().unwrap().steps()[1].clone();
    let mut form = build::survey_step("Survey", 1, &step);
    form.set_selected("completion".to_owned());

    assert_eq!(apply(&mut protocol, &form), Applied::Changed);

    let survey = protocol.task("Survey").unwrap().survey().unwrap();
    assert_eq!(survey.steps()[1].identifier(), "completion");
    assert_eq!(
        survey.navigation_rules().unwrap()["intro"].destinations(),
        ["completion"],
        "the branch followed the rename"
    );
    assert!(errors(&protocol).is_empty(), "{:?}", errors(&protocol));
}

/// Two steps sharing an identifier would record their answers under one key.
#[test]
fn a_duplicate_step_identifier_is_refused() {
    let mut protocol = protocol();
    let survey = protocol
        .tasks
        .iter_mut()
        .find(|task| task.name() == "Survey")
        .and_then(carp_protocol::task::Task::survey_mut)
        .unwrap();
    survey
        .steps_mut()
        .unwrap()
        .push(carp_protocol::survey::RpStep::instruction(
            "outro",
            "Thanks",
            "All done.",
        ));

    let step = protocol.task("Survey").unwrap().survey().unwrap().steps()[1].clone();
    let mut form = build::survey_step("Survey", 1, &step);
    form.set_selected("intro".to_owned());

    assert!(matches!(
        apply(&mut protocol, &form),
        Applied::Refused(reason) if reason.contains("already uses")
    ));
}

/// A form naming something deleted in another pane must say so rather than
/// resurrecting it or panicking.
#[test]
fn a_form_for_something_deleted_reports_it_vanished() {
    let mut protocol = protocol();
    let form = build::device(protocol.device("Location Service").unwrap());
    builder::remove_device(&mut protocol, "Location Service");

    assert_eq!(apply(&mut protocol, &form), Applied::Vanished);

    // The same for a trigger id that was never in the protocol.
    let absent = build::trigger(99, &carp_protocol::Trigger::immediate("x"), &protocol);
    assert_eq!(apply(&mut protocol, &absent), Applied::Vanished);
}

/// Zero minutes and a zero expiry mean "unstated" and have to be written by
/// omitting the field, not by storing a zero the app would render.
#[test]
fn zero_stands_for_absent_on_the_wire() {
    let mut protocol = protocol();
    let mut form = build::task(protocol.task("Survey").unwrap());

    for key in ["minutes_to_complete", "expire"] {
        form.selected = form
            .fields
            .iter()
            .position(|field| field.key == key)
            .unwrap();
        assert!(form.begin_typing());
        form.clear_buffer();
        for character in "0".chars() {
            form.push(character);
        }
        assert!(form.commit(), "{:?}", form.error);
    }

    assert_eq!(apply(&mut protocol, &form), Applied::Changed);

    let written = serde_json::to_value(protocol.task("Survey").unwrap()).unwrap();
    assert!(written.get("minutesToComplete").is_none(), "got {written}");
    assert!(written.get("expire").is_none(), "got {written}");
}

/// A protocol without study-app settings gains them the first time they are
/// edited, rather than the edit being dropped.
#[test]
fn editing_study_app_settings_creates_the_block_when_absent() {
    let mut protocol = protocol();
    protocol.application_data = None;

    let mut form = build::application_data(&protocol);
    form.selected = form
        .fields
        .iter()
        .position(|field| field.key == "title")
        .unwrap();
    form.set_selected("study.description.title".to_owned());

    assert_eq!(apply(&mut protocol, &form), Applied::Changed);
    let data = protocol
        .application_data
        .as_ref()
        .expect("the block was created");
    assert_eq!(
        data.study_description.as_ref().unwrap().title,
        "study.description.title"
    );
}

/// A cron expression with the wrong number of fields would silently never
/// fire, so it is refused where it is typed.
#[test]
fn a_malformed_cron_expression_is_refused() {
    let mut protocol = protocol();
    let id = builder::add_trigger(&mut protocol, TriggerKind::CronScheduled, "Primary Phone");
    let trigger = protocol.triggers[&id].clone();

    let mut form = build::trigger(id, &trigger, &protocol);
    form.selected = form
        .fields
        .iter()
        .position(|field| field.key == "cron")
        .unwrap();
    form.set_selected("0 10 *".to_owned());

    assert!(matches!(
        apply(&mut protocol, &form),
        Applied::Refused(reason) if reason.contains("five fields")
    ));
}

/// Validation errors reported for a protocol, as readable lines.
fn errors(protocol: &StudyProtocol) -> Vec<String> {
    carp_protocol::validate(protocol)
        .into_iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
        .map(|diagnostic| format!("[{}] {}", diagnostic.location, diagnostic.message))
        .collect()
}

/// Every subject has to produce a title, since the form's header shows it.
#[test]
fn every_subject_has_a_title() {
    let subjects = [
        Subject::Protocol,
        Subject::ApplicationData,
        Subject::Device("Phone".to_owned()),
        Subject::Task("Survey".to_owned()),
        Subject::Trigger(3),
        Subject::ParticipantRole("Participant".to_owned()),
        Subject::ExpectedData(0),
        Subject::SurveyStep {
            task: "Survey".to_owned(),
            step: 0,
        },
        Subject::Measure {
            task: "Survey".to_owned(),
            measure: 0,
        },
    ];
    for subject in subjects {
        assert!(!subject.title().is_empty(), "{subject:?}");
    }
}
