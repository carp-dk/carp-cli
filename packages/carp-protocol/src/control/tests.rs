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
fn a_task_control_round_trips() {
    let original = serde_json::json!({
        "triggerId": 1,
        "taskName": "Neuropathy Assessment",
        "destinationDeviceRoleName": "Neuropathy Tracker",
        "control": "Start"
    });

    let control: TaskControl = serde_json::from_value(original.clone()).unwrap();
    assert_eq!(control.trigger_id, 1);
    assert_eq!(serde_json::to_value(&control).unwrap(), original);
}

/// `Start` and `Stop` are written capitalised, as Kotlin enum names.
#[test]
fn the_control_verb_keeps_its_capital() {
    assert_eq!(
        serde_json::to_value(Control::Start).unwrap(),
        serde_json::json!("Start")
    );
    assert_eq!(
        serde_json::to_value(Control::Stop).unwrap(),
        serde_json::json!("Stop")
    );
}

#[test]
fn a_device_connection_round_trips() {
    let original = serde_json::json!({
        "roleName": "Polar HR Device",
        "connectedToRoleName": "Primary Phone"
    });

    let connection: DeviceConnection = serde_json::from_value(original.clone()).unwrap();
    assert_eq!(connection.connected_to_role_name, "Primary Phone");
    assert_eq!(serde_json::to_value(&connection).unwrap(), original);
}
