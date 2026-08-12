// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`], the module that keeps a protocol's references intact.
//!
//! Each test asserts the same thing in a different place: after the edit, the
//! protocol still validates with no errors. That is the property the module
//! exists for, and it is stronger than checking individual fields, because a
//! reference left dangling anywhere shows up as a validation error.

use super::*;
use crate::device::DeviceKind;
use crate::protocol::StudyProtocol;
use crate::task::TaskKind;
use crate::trigger::TriggerKind;
use crate::validate::{Severity, validate};

/// A protocol with a phone, a connected wearable, and one survey started by a
/// daily trigger - the shape most studies have.
fn protocol() -> StudyProtocol {
    let mut protocol = StudyProtocol::new("Sleep", "979b408d-784e-4b1b-bb1e-ff9204e072f3");
    let phone = add_device(&mut protocol, DeviceKind::Smartphone);
    add_device(&mut protocol, DeviceKind::PolarDevice);
    add_task(
        &mut protocol,
        TaskKind::RpApp,
        &phone,
        TriggerKind::RecurrentScheduled,
    );
    add_participant_role(&mut protocol, "Participant");
    protocol
        .expected_participant_data
        .push(crate::participant::ExpectedParticipantData::for_roles(
            "dk.carp.webservices.input.informed_consent",
            vec!["Participant".to_owned()],
        ));
    protocol
}

/// Errors reported for `protocol`, as readable lines.
fn errors(protocol: &StudyProtocol) -> Vec<String> {
    validate(protocol)
        .into_iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
        .map(|diagnostic| format!("[{}] {}", diagnostic.location, diagnostic.message))
        .collect()
}

/// The fixture itself has to be sound, or every test below proves nothing.
#[test]
fn the_starting_protocol_is_valid() {
    assert_eq!(errors(&protocol()), Vec::<String>::new());
}

/// Adding a connected device without a connection leaves it unreachable, so
/// the connection is made at the same time.
#[test]
fn a_connected_device_is_wired_to_a_primary_device() {
    let protocol = protocol();
    let connection = protocol
        .connections
        .iter()
        .find(|connection| connection.role_name == "Polar HR Device")
        .expect("the wearable is connected");
    assert_eq!(connection.connected_to_role_name, "Primary Phone");
    assert_eq!(errors(&protocol), Vec::<String>::new());
}

/// A second device of the same kind must not take the first one's role name,
/// which would silently re-point every reference to it.
#[test]
fn a_second_device_of_a_kind_gets_its_own_name() {
    let mut protocol = protocol();
    let second = add_device(&mut protocol, DeviceKind::Smartphone);

    assert_eq!(second, "Primary Phone 2");
    assert_eq!(protocol.device_role_names().len(), 3);
    assert_eq!(errors(&protocol), Vec::<String>::new());
}

/// Renaming a device must reach the triggers, controls and connections that
/// name it.
#[test]
fn renaming_a_device_moves_every_reference() {
    let mut protocol = protocol();
    assert!(rename_device(&mut protocol, "Primary Phone", "Study Phone"));

    assert!(protocol.device("Study Phone").is_some());
    assert!(protocol.device("Primary Phone").is_none());
    for trigger in protocol.triggers.values() {
        assert_ne!(trigger.source_device(), "Primary Phone");
    }
    for control in &protocol.task_controls {
        assert_ne!(control.destination_device_role_name, "Primary Phone");
    }
    for connection in &protocol.connections {
        assert_ne!(connection.connected_to_role_name, "Primary Phone");
    }
    assert_eq!(errors(&protocol), Vec::<String>::new());
}

/// Renaming onto a name already in use would produce two devices with one
/// role name, so it is refused rather than performed.
#[test]
fn renaming_a_device_onto_a_taken_name_is_refused() {
    let mut protocol = protocol();
    assert!(!rename_device(
        &mut protocol,
        "Primary Phone",
        "Polar HR Device"
    ));
    assert!(protocol.device("Primary Phone").is_some());
    assert_eq!(errors(&protocol), Vec::<String>::new());
}

/// Removing the device a trigger fires on has to take the trigger and its
/// controls too, since a trigger cannot run on a device that is gone.
#[test]
fn removing_a_device_takes_its_triggers_and_controls() {
    let mut protocol = protocol();
    let removal = remove_device(&mut protocol, "Primary Phone");

    assert_eq!(removal.triggers, 1);
    assert_eq!(removal.task_controls, 1);
    assert_eq!(removal.connections, 1, "the wearable hung off this phone");
    assert!(protocol.triggers.is_empty());
    assert!(protocol.task_controls.is_empty());

    // The task survives, since it can be re-pointed at another device. What
    // is left is a warning, not a dangling reference.
    assert_eq!(protocol.tasks.len(), 1);
    assert_eq!(
        errors(&protocol),
        ["[devices] the protocol has no primary device"]
    );
}

/// Renaming a task must reach the controls that start it and the triggers
/// that watch it.
#[test]
fn renaming_a_task_moves_every_reference() {
    let mut protocol = protocol();
    let watcher = add_trigger(&mut protocol, TriggerKind::UserTask, "Primary Phone");
    protocol
        .triggers
        .get_mut(&watcher)
        .unwrap()
        .set_watched_task("Survey");
    add_task_control(&mut protocol, watcher, "Survey", "Primary Phone");

    assert!(rename_task(&mut protocol, "Survey", "Sleep Diary"));

    assert!(protocol.task("Sleep Diary").is_some());
    assert_eq!(
        protocol.triggers[&watcher].watched_task(),
        Some("Sleep Diary")
    );
    for control in &protocol.task_controls {
        assert_eq!(control.task_name, "Sleep Diary");
    }
    assert_eq!(errors(&protocol), Vec::<String>::new());
}

/// Removing a task must take the controls that started it, and any trigger
/// those controls were the only use of.
#[test]
fn removing_a_task_takes_its_controls_and_orphaned_triggers() {
    let mut protocol = protocol();
    let removal = remove_task(&mut protocol, "Survey");

    assert_eq!(removal.tasks, 1);
    assert_eq!(removal.task_controls, 1);
    assert_eq!(removal.triggers, 1, "nothing else used that trigger");
    assert_eq!(removal.summary(), "1 task, 1 trigger, 1 task control");
    assert_eq!(errors(&protocol), Vec::<String>::new());
}

/// A trigger shared by two tasks must survive the removal of one of them.
#[test]
fn a_shared_trigger_survives_removing_one_of_its_tasks() {
    let mut protocol = protocol();
    let trigger_id = *protocol.triggers.keys().next().unwrap();
    let second = add_task(
        &mut protocol,
        TaskKind::Background,
        "Primary Phone",
        TriggerKind::NoOp,
    );
    add_task_control(&mut protocol, trigger_id, &second, "Primary Phone");

    let removal = remove_task(&mut protocol, &second);

    assert_eq!(removal.tasks, 1);
    assert!(
        protocol.triggers.contains_key(&trigger_id),
        "the shared trigger still starts the survey"
    );
    assert_eq!(errors(&protocol), Vec::<String>::new());
}

/// Attaching the same task to the same trigger twice would start it twice.
#[test]
fn a_duplicate_task_control_is_not_added() {
    let mut protocol = protocol();
    let trigger_id = *protocol.triggers.keys().next().unwrap();

    assert!(!add_task_control(
        &mut protocol,
        trigger_id,
        "Survey",
        "Primary Phone"
    ));
    assert_eq!(protocol.task_controls.len(), 1);
}

/// Renaming a role has to follow into the expected data assigned to it, or
/// the study asks a role that no longer exists for its consent.
#[test]
fn renaming_a_role_moves_its_expected_data() {
    let mut protocol = protocol();
    assert!(rename_participant_role(
        &mut protocol,
        "Participant",
        "Patient"
    ));

    assert_eq!(protocol.participant_roles[0].role, "Patient");
    assert_eq!(
        protocol.expected_participant_data[0]
            .assigned_to
            .role_names()
            .unwrap(),
        ["Patient"]
    );
    assert_eq!(errors(&protocol), Vec::<String>::new());
}

/// Removing a role must not leave expected data assigned to it. Data assigned
/// to several roles keeps the others.
#[test]
fn removing_a_role_prunes_its_expected_data() {
    let mut protocol = protocol();
    add_participant_role(&mut protocol, "Clinician");
    protocol
        .expected_participant_data
        .push(crate::participant::ExpectedParticipantData::for_roles(
            "dk.cachet.carp.input.sex",
            vec!["Participant".to_owned(), "Clinician".to_owned()],
        ));

    remove_participant_role(&mut protocol, "Clinician");

    assert_eq!(protocol.participant_roles.len(), 1);
    // The consent entry named only the removed role's peer, so it stays as is.
    let shared = &protocol.expected_participant_data[1];
    assert_eq!(shared.assigned_to.role_names().unwrap(), ["Participant"]);
    assert_eq!(errors(&protocol), Vec::<String>::new());
}

/// Expected data that named only the removed role has nothing left to apply
/// to, so it goes with the role.
#[test]
fn removing_a_role_drops_expected_data_only_it_had() {
    let mut protocol = protocol();
    remove_participant_role(&mut protocol, "Participant");

    assert!(protocol.participant_roles.is_empty());
    assert!(protocol.expected_participant_data.is_empty());
    assert_eq!(errors(&protocol), Vec::<String>::new());
}

#[test]
fn unique_names_count_up_from_the_base() {
    let taken = vec!["Phone".to_owned(), "Phone 2".to_owned()];
    assert_eq!(unique_name("Phone", &taken), "Phone 3");
    assert_eq!(unique_name("Watch", &taken), "Watch");
    assert_eq!(unique_name("  ", &[]), "Unnamed");
}

/// The whole point of the module, stated once: a long sequence of edits must
/// leave a protocol with no dangling references.
#[test]
fn a_sequence_of_edits_never_breaks_the_graph() {
    let mut protocol = protocol();

    let watch = add_device(&mut protocol, DeviceKind::MovesenseDevice);
    add_task(&mut protocol, TaskKind::Background, &watch, TriggerKind::Immediate);
    rename_device(&mut protocol, &watch, "Chest Sensor");
    rename_task(&mut protocol, "Survey", "Morning Diary");
    add_task(
        &mut protocol,
        TaskKind::HealthApp,
        "Primary Phone",
        TriggerKind::Periodic,
    );
    remove_device(&mut protocol, "Polar HR Device");
    add_participant_role(&mut protocol, "Participant");
    remove_task(&mut protocol, "Morning Diary");
    rename_participant_role(&mut protocol, "Participant", "Volunteer");

    assert_eq!(errors(&protocol), Vec::<String>::new());
}
