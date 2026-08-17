// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`] and [`crate::app::form::build`], which are two halves
//! of one thing and so are tested together.
//!
//! The property that matters most is stated once in
//! [`opening_and_submitting_every_form_unchanged_changes_nothing`]: build a
//! form, submit it without touching it, and the protocol must be byte-identical.
//! Anything the build forgot to read, or the apply forgot to write, breaks it.

use carp_protocol::device::DeviceKind;
use carp_protocol::task::TaskKind;
use carp_protocol::trigger::TriggerKind;
use carp_protocol::{StudyProtocol, builder};

use super::{Applied, apply};
use crate::app::form::build;

/// A protocol exercising every form: a phone, a location service, a survey,
/// a health task, a scheduled trigger, a watching trigger and a role.
pub(super) fn protocol() -> StudyProtocol {
    let mut protocol = StudyProtocol::new("Sleep", "979b408d-784e-4b1b-bb1e-ff9204e072f3");
    let phone = builder::add_device(&mut protocol, DeviceKind::Smartphone);
    builder::add_device(&mut protocol, DeviceKind::LocationService);
    builder::add_device(&mut protocol, DeviceKind::CortriumDevice);

    builder::add_task(
        &mut protocol,
        TaskKind::RpApp,
        &phone,
        TriggerKind::RecurrentScheduled,
    );
    builder::add_task(
        &mut protocol,
        TaskKind::HealthApp,
        &phone,
        TriggerKind::Periodic,
    );
    builder::add_task(
        &mut protocol,
        TaskKind::Background,
        &phone,
        TriggerKind::Immediate,
    );

    let watcher = builder::add_trigger(&mut protocol, TriggerKind::UserTask, &phone);
    protocol
        .triggers
        .get_mut(&watcher)
        .unwrap()
        .set_watched_task("Survey");
    builder::add_task_control(&mut protocol, watcher, "Health Task", &phone);

    builder::add_participant_role(&mut protocol, "Participant");
    protocol.expected_participant_data.push(
        carp_protocol::participant::ExpectedParticipantData::for_roles(
            "dk.carp.webservices.input.informed_consent",
            vec!["Participant".to_owned()],
        ),
    );

    // Give the survey a step, so the step form has something to open.
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
            "intro",
            "Welcome",
            "Please answer honestly.",
        ));

    protocol
}

/// Every form the editor can open on this protocol.
pub(super) fn all_forms(protocol: &StudyProtocol) -> Vec<crate::app::form::Form> {
    let mut forms = vec![build::protocol(protocol), build::application_data(protocol)];

    forms.extend(protocol.devices().map(build::device));
    forms.extend(protocol.tasks.iter().map(build::task));
    forms.extend(
        protocol
            .triggers
            .iter()
            .map(|(id, trigger)| build::trigger(*id, trigger, protocol)),
    );
    forms.extend(
        protocol
            .participant_roles
            .iter()
            .map(build::participant_role),
    );
    forms.extend(
        protocol
            .expected_participant_data
            .iter()
            .enumerate()
            .map(|(index, expected)| build::expected_data(index, expected, protocol)),
    );
    for task in &protocol.tasks {
        if let Some(survey) = task.survey() {
            forms.extend(
                survey
                    .steps()
                    .iter()
                    .enumerate()
                    .map(|(index, step)| build::survey_step(task.name(), index, step)),
            );
        }
        forms.extend(
            task.measures()
                .iter()
                .enumerate()
                .map(|(index, measure)| build::measure(task.name(), index, measure)),
        );
    }
    forms
}

/// The central property: opening a form and submitting it untouched must
/// leave the protocol exactly as it was.
///
/// A field the build never read comes back as its default and overwrites the
/// real value; a field the apply never wrote silently discards the edit. Both
/// show up here.
#[test]
fn opening_and_submitting_every_form_unchanged_changes_nothing() {
    let original = protocol();

    for form in all_forms(&original) {
        let mut protocol = original.clone();
        let outcome = apply(&mut protocol, &form);

        assert_eq!(
            outcome,
            Applied::Changed,
            "{:?} was not applied",
            form.subject
        );
        assert_eq!(
            serde_json::to_value(&protocol).unwrap(),
            serde_json::to_value(&original).unwrap(),
            "submitting the {:?} form unchanged altered the protocol",
            form.subject
        );
    }
}
