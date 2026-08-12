// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Adding, renaming and removing devices.
//!
//! See [`super`] for why every mutation goes through this module.

use crate::control::DeviceConnection;
use crate::device::{Device, DeviceKind};
use crate::protocol::StudyProtocol;

use super::{Removal, unique_name};

/// Add a device of `kind`, giving it a role name not already in use.
///
/// Connected devices are wired to the first primary device, because a
/// connected device with no connection is never reached and there is no other
/// sensible default.
///
/// Returns the role name it settled on.
pub fn add_device(protocol: &mut StudyProtocol, kind: DeviceKind) -> String {
    let role_name = unique_name(kind.default_role_name(), &protocol.device_role_names());
    let device = Device::new(kind, role_name.clone());

    if kind.is_primary() {
        protocol.primary_devices.push(device);
    } else {
        protocol.connected_devices.push(device);
        if let Some(primary) = protocol.primary_devices.first() {
            let primary = primary.role_name().to_owned();
            protocol
                .connections
                .push(DeviceConnection::new(role_name.clone(), primary));
        }
    }
    role_name
}

/// Rename a device and every reference to it.
///
/// Does nothing if `to` is already taken, since two devices sharing a role
/// name is exactly the state this module exists to prevent.
pub fn rename_device(protocol: &mut StudyProtocol, from: &str, to: &str) -> bool {
    if from == to {
        return true;
    }
    if to.trim().is_empty() || protocol.device(to).is_some() {
        return false;
    }

    let Some(device) = protocol
        .primary_devices
        .iter_mut()
        .chain(&mut protocol.connected_devices)
        .find(|device| device.role_name() == from)
    else {
        return false;
    };
    device.set_role_name(to);

    for trigger in protocol.triggers.values_mut() {
        if trigger.source_device() == from {
            trigger.set_source_device(to);
        }
    }
    for control in &mut protocol.task_controls {
        if control.destination_device_role_name == from {
            to.clone_into(&mut control.destination_device_role_name);
        }
    }
    for connection in &mut protocol.connections {
        if connection.role_name == from {
            to.clone_into(&mut connection.role_name);
        }
        if connection.connected_to_role_name == from {
            to.clone_into(&mut connection.connected_to_role_name);
        }
    }
    for devices in protocol.assigned_devices.values_mut() {
        if devices.remove(from) {
            devices.insert(to.to_owned());
        }
    }
    true
}

/// Remove a device, along with the connections, triggers and task controls
/// that could only have referred to it.
///
/// Tasks are left in place: a task is not owned by a device, and re-pointing
/// it at another device is usually what is wanted. The task controls that
/// named this device do go, so the caller is told how many.
pub fn remove_device(protocol: &mut StudyProtocol, role_name: &str) -> Removal {
    let mut removal = Removal::default();

    protocol
        .primary_devices
        .retain(|device| device.role_name() != role_name);
    protocol
        .connected_devices
        .retain(|device| device.role_name() != role_name);

    let before = protocol.connections.len();
    protocol.connections.retain(|connection| {
        connection.role_name != role_name && connection.connected_to_role_name != role_name
    });
    removal.connections = before - protocol.connections.len();

    let before = protocol.task_controls.len();
    protocol
        .task_controls
        .retain(|control| control.destination_device_role_name != role_name);
    removal.task_controls = before - protocol.task_controls.len();

    // A trigger fires on exactly one device, so one that fired on this device
    // has nowhere left to run.
    let doomed: Vec<u32> = protocol
        .triggers
        .iter()
        .filter(|(_, trigger)| trigger.source_device() == role_name)
        .map(|(id, _)| *id)
        .collect();
    removal.triggers = doomed.len();
    for id in doomed {
        protocol.triggers.remove(&id);
        let before = protocol.task_controls.len();
        protocol.task_controls.retain(|control| control.trigger_id != id);
        removal.task_controls += before - protocol.task_controls.len();
    }

    removal
}
