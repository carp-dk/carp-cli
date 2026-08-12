// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`].

use super::*;
use carp_protocol::{DeviceKind, builder, task::TaskKind, trigger::TriggerKind};

fn protocol() -> StudyProtocol {
    let mut protocol = StudyProtocol::new("Test", "979b408d-784e-4b1b-bb1e-ff9204e072f3");
    let phone = builder::add_device(&mut protocol, DeviceKind::Smartphone);
    builder::add_device(&mut protocol, DeviceKind::PolarDevice);
    builder::add_task(&mut protocol, TaskKind::RpApp, &phone, TriggerKind::Immediate);
    builder::add_task(&mut protocol, TaskKind::Background, &phone, TriggerKind::NoOp);
    builder::add_participant_role(&mut protocol, "Participant");
    protocol
}

/// A fresh list selects the first row of anything non-empty, so a detail
/// panel never sits blank next to a full table.
#[test]
fn syncing_selects_the_first_row() {
    let mut lists = Lists::default();
    lists.sync(&protocol(), None);

    assert_eq!(lists.devices.selected(), Some(0));
    assert_eq!(lists.tasks.selected(), Some(0));
    assert_eq!(lists.roles.selected(), Some(0));
}

/// A cursor left past the end of a shortened list would index nothing.
#[test]
fn syncing_pulls_a_stale_cursor_back() {
    let mut protocol = protocol();
    let mut lists = Lists::default();
    lists.sync(&protocol, None);
    lists.devices.select(Some(1));

    builder::remove_device(&mut protocol, "Polar HR Device");
    lists.sync(&protocol, None);

    assert_eq!(lists.devices.selected(), Some(0));
    assert!(lists.selected_device(&protocol).is_some());
}

/// An emptied list has no selection at all, rather than row zero of
/// nothing.
#[test]
fn an_empty_list_has_no_cursor() {
    let mut protocol = protocol();
    protocol.participant_roles.clear();

    let mut lists = Lists::default();
    lists.sync(&protocol, None);
    assert_eq!(lists.roles.selected(), None);
    assert!(lists.selected_role(&protocol).is_none());
}

/// Devices are listed primary-first, so the cursor has to read them in
/// that order rather than out of either underlying list.
#[test]
fn devices_are_indexed_primary_first() {
    let protocol = protocol();
    let mut lists = Lists::default();
    lists.sync(&protocol, None);

    assert_eq!(lists.selected_device_role(&protocol).unwrap(), "Primary Phone");
    lists.devices.select(Some(1));
    assert_eq!(
        lists.selected_device_role(&protocol).unwrap(),
        "Polar HR Device"
    );
}

/// Triggers live in a map; the cursor indexes their id order.
#[test]
fn triggers_are_indexed_in_id_order() {
    let protocol = protocol();
    let mut lists = Lists::default();
    lists.sync(&protocol, None);

    assert_eq!(lists.selected_trigger_id(&protocol), Some(0));
    lists.triggers.select(Some(1));
    assert_eq!(lists.selected_trigger_id(&protocol), Some(1));
}

/// Moving between tasks has to reset the measures cursor, or the pane
/// describes the previous task's measure.
#[test]
fn changing_task_resets_the_measure_cursor() {
    let protocol = protocol();
    let mut lists = Lists::default();
    lists.sync(&protocol, None);
    lists.measures.select(Some(0));

    lists.move_in(Section::Tasks, &protocol, None, 0, 0, 1);

    assert_eq!(lists.tasks.selected(), Some(1));
    // The background task has no measures, so nothing is selected.
    assert_eq!(lists.measures.selected(), None);
}
