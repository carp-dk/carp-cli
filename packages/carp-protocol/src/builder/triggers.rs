// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Adding and removing triggers, and wiring them to tasks.
//!
//! See [`super`] for why every mutation goes through this module.

use crate::control::TaskControl;
use crate::protocol::StudyProtocol;
use crate::trigger::TriggerKind;

use super::Removal;

/// Add a trigger of `kind` on `device`, returning its id.
pub fn add_trigger(protocol: &mut StudyProtocol, kind: TriggerKind, device: &str) -> u32 {
    let id = protocol.next_trigger_id();
    protocol.triggers.insert(id, kind.instantiate(device.to_owned()));
    id
}

/// Remove a trigger and the task controls it drove.
pub fn remove_trigger(protocol: &mut StudyProtocol, trigger_id: u32) -> Removal {
    let mut removal = Removal::default();
    if protocol.triggers.remove(&trigger_id).is_some() {
        removal.triggers = 1;
    }
    let before = protocol.task_controls.len();
    protocol
        .task_controls
        .retain(|control| control.trigger_id != trigger_id);
    removal.task_controls = before - protocol.task_controls.len();
    removal
}

/// Attach `task_name` to `trigger_id`, to run on `device`.
///
/// Adding the same triple twice is a no-op rather than a duplicate.
pub fn add_task_control(
    protocol: &mut StudyProtocol,
    trigger_id: u32,
    task_name: &str,
    device: &str,
) -> bool {
    let exists = protocol.task_controls.iter().any(|control| {
        control.trigger_id == trigger_id
            && control.task_name == task_name
            && control.destination_device_role_name == device
    });
    if exists {
        return false;
    }
    protocol
        .task_controls
        .push(TaskControl::start(trigger_id, task_name, device));
    true
}
