// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`], the editor's own state.

use super::*;
use carp_protocol::DeviceKind;

fn studio() -> Studio {
    Studio::new("979b408d-784e-4b1b-bb1e-ff9204e072f3".to_owned())
}

/// A protocol read from a file keeps its identity: opening and saving must
/// not turn a study's protocol into a different one.
#[test]
fn opening_a_protocol_keeps_its_identity() {
    let original: StudyProtocol = serde_json::from_str(include_str!(
        "../../packages/carp-protocol/tests/corpus/neuropathy.json"
    ))
    .unwrap();

    let studio = Studio::opened(original.clone(), None);

    assert_eq!(studio.protocol.id, original.id);
    assert_eq!(studio.protocol.version, original.version);
    assert!(!studio.dirty, "opening is not a change");
    assert!(studio.history.is_empty(), "there is nothing to undo yet");
}

/// A real protocol has to survive being opened without the editor deciding it
/// is broken.
#[test]
fn a_reference_protocol_opens_without_errors() {
    let protocol: StudyProtocol = serde_json::from_str(include_str!(
        "../../packages/carp-protocol/tests/corpus/neuropathy.json"
    ))
    .unwrap();

    let studio = Studio::opened(protocol, None);
    let (errors, _, _) = studio.check_counts();
    assert_eq!(errors, 0);
}

/// The header has to show whether there is unsaved work, since that is the
/// only warning before quitting.
#[test]
fn the_location_marks_unsaved_work() {
    let mut studio = studio();
    assert_eq!(studio.location(), "unsaved");

    studio.dirty = true;
    assert_eq!(studio.location(), "unsaved *");

    studio.path = Some(std::path::PathBuf::from("/tmp/study/protocol.json"));
    assert_eq!(studio.location(), "protocol.json *");

    studio.dirty = false;
    assert_eq!(studio.location(), "protocol.json");
}

/// A form submitted with nothing changed must not create an undo step, or
/// pressing undo would appear to do nothing.
#[test]
fn an_unchanged_form_records_no_undo_step() {
    let mut studio = studio();
    studio.form = Some(crate::app::form::build::device(
        studio.protocol.device("Primary Phone").unwrap(),
    ));

    assert_eq!(studio.submit_form(), None);
    assert!(studio.history.is_empty());
    assert!(!studio.dirty);
}

/// A refused submission must not leave a half-applied undo step behind.
#[test]
fn a_refused_submission_leaves_no_undo_step() {
    let mut studio = studio();
    let mut form = crate::app::form::build::device(
        studio.protocol.device("Primary Phone").unwrap(),
    );
    form.set_selected(String::new());
    studio.form = Some(form);

    assert!(studio.submit_form().is_some(), "an empty role name is refused");
    assert!(studio.history.is_empty());
    assert!(studio.form.is_some(), "the form stays open");
}

/// The Survey tab falls back to the first task that has a survey, so it is
/// never blank while the protocol contains one.
#[test]
fn the_survey_tab_falls_back_to_the_first_survey() {
    let mut studio = studio();
    assert_eq!(studio.survey_task_name(), None);

    carp_protocol::builder::add_task(
        &mut studio.protocol,
        carp_protocol::task::TaskKind::RpApp,
        "Primary Phone",
        carp_protocol::trigger::TriggerKind::Immediate,
    );
    assert_eq!(studio.survey_task_name().as_deref(), Some("Survey"));

    // A stale name from a deleted task does not stick.
    studio.survey_task = Some("Removed".to_owned());
    assert_eq!(studio.survey_task_name().as_deref(), Some("Survey"));
}

/// The checks have to follow the protocol, or the Checks tab describes a
/// protocol that no longer exists.
#[test]
fn the_checks_follow_a_change() {
    let mut studio = studio();
    assert_eq!(studio.check_counts().0, 0);

    carp_protocol::builder::remove_device(&mut studio.protocol, "Primary Phone");
    studio.changed();

    assert_eq!(
        studio.check_counts().0,
        1,
        "a protocol with no primary device has an error"
    );
}

/// Adding a device the editor cannot connect must still leave a valid
/// protocol, since the connection is made for it.
#[test]
fn a_connected_device_is_usable_immediately() {
    let mut studio = studio();
    studio.checkpoint();
    carp_protocol::builder::add_device(&mut studio.protocol, DeviceKind::HealthService);
    studio.changed();

    assert_eq!(studio.check_counts().0, 0);
    assert_eq!(studio.protocol.connections.len(), 1);
}
