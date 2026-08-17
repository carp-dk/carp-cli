// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Whether the devices are named, placed and reachable.

use std::collections::HashSet;

use super::super::Diagnostic;
use crate::device::Device;
use crate::protocol::StudyProtocol;

/// Devices have unique, non-empty role names and sit in the right list.
pub fn devices(protocol: &StudyProtocol, out: &mut Vec<Diagnostic>) {
    if protocol.primary_devices.is_empty() {
        out.push(
            Diagnostic::error("devices", "the protocol has no primary device")
                .with_hint("add a Smartphone: a deployment is created per primary device"),
        );
    }

    let mut seen: HashSet<&str> = HashSet::new();
    for device in protocol.devices() {
        let role = device.role_name();
        if role.trim().is_empty() {
            out.push(Diagnostic::error(
                format!("device <{}>", device.type_label()),
                "has no role name",
            ));
        } else if !seen.insert(role) {
            out.push(
                Diagnostic::error(format!("device {role:?}"), "role name is used twice")
                    .with_hint("triggers and task controls address devices by role name"),
            );
        }
    }

    // A kind that belongs in the other list is a real fault: CAWS creates a
    // deployment per primary device, so a service listed as primary produces a
    // deployment nothing can register.
    for device in &protocol.primary_devices {
        if let Some(kind) = device.kind()
            && !kind.is_primary()
        {
            out.push(
                Diagnostic::error(
                    format!("device {:?}", device.role_name()),
                    format!("{} is not a primary device", kind.label()),
                )
                .with_hint("move it to the connected devices"),
            );
        }
    }
    for device in &protocol.connected_devices {
        if let Some(kind) = device.kind()
            && kind.is_primary()
        {
            out.push(
                Diagnostic::error(
                    format!("device {:?}", device.role_name()),
                    format!("{} is a primary device", kind.label()),
                )
                .with_hint("move it to the primary devices"),
            );
        }
        if device.kind().is_some_and(|kind| kind.needs_api_key()) && api_key(device).is_none() {
            out.push(
                Diagnostic::warning(format!("device {:?}", device.role_name()), "has no API key")
                    .with_hint("the service returns nothing without one"),
            );
        }
    }
}

/// The API key of a service device, when it has a non-empty one.
fn api_key(device: &Device) -> Option<&str> {
    use crate::device::KnownDevice;
    let Device::Known(known) = device else {
        return None;
    };
    let key = match known.as_ref() {
        KnownDevice::WeatherService { api_key, .. }
        | KnownDevice::AirQualityService { api_key, .. } => api_key.as_str(),
        _ => return None,
    };
    (!key.trim().is_empty()).then_some(key)
}

/// Every connected device is reachable, and every connection resolves.
pub fn connections(protocol: &StudyProtocol, out: &mut Vec<Diagnostic>) {
    let primaries: HashSet<&str> = protocol
        .primary_devices
        .iter()
        .map(Device::role_name)
        .collect();
    let connected: HashSet<&str> = protocol
        .connected_devices
        .iter()
        .map(Device::role_name)
        .collect();

    for connection in &protocol.connections {
        if !connected.contains(connection.role_name.as_str()) {
            out.push(Diagnostic::error(
                format!("connection {:?}", connection.role_name),
                "names a device that is not a connected device",
            ));
        }
        if !primaries.contains(connection.connected_to_role_name.as_str()) {
            out.push(Diagnostic::error(
                format!("connection {:?}", connection.role_name),
                format!(
                    "connects to {:?}, which is not a primary device",
                    connection.connected_to_role_name
                ),
            ));
        }
    }

    // A connected device with no connection is never reached, so whatever it
    // measures is never collected.
    let wired: HashSet<&str> = protocol
        .connections
        .iter()
        .map(|connection| connection.role_name.as_str())
        .collect();
    for device in &protocol.connected_devices {
        if !wired.contains(device.role_name()) {
            out.push(
                Diagnostic::warning(
                    format!("device {:?}", device.role_name()),
                    "is not connected to a primary device",
                )
                .with_hint("add a connection, or the device is never reached"),
            );
        }
    }
}
