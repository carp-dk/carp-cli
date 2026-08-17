// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Applying a device form.

use carp_protocol::StudyProtocol;
use carp_protocol::builder;
use carp_protocol::device::{Device, KnownDevice};

use crate::app::form::Form;

use super::Applied;

/// Write a device form back, renaming through [`builder`] so that every
/// trigger, task control and connection naming the device moves with it.
pub fn apply(protocol: &mut StudyProtocol, form: &Form, role: &str) -> Applied {
    if protocol.device(role).is_none() {
        return Applied::Vanished;
    }

    // Rename first: everything after addresses the device by its new name.
    let new_role = form.text("role_name");
    if new_role.trim().is_empty() {
        return Applied::Refused("a device needs a role name".to_owned());
    }
    if new_role != role && !builder::rename_device(protocol, role, &new_role) {
        return Applied::Refused(format!("another device is already called {new_role:?}"));
    }

    let optional = form.flag("is_optional");
    let Some(device) = protocol
        .primary_devices
        .iter_mut()
        .chain(&mut protocol.connected_devices)
        .find(|device| device.role_name() == new_role)
    else {
        return Applied::Vanished;
    };

    let Device::Known(known) = device else {
        // An unmodelled device has no typed fields to write; its role name
        // was still renamed above, which is all the form offered.
        return Applied::Changed;
    };
    known.core_mut().is_optional = Some(optional);

    // The settings shared across namespaces, written through the accessors
    // so a CAMS 2.0 device is handled exactly as its older counterpart.
    if let Some(location) = known.location_mut() {
        location.accuracy = form.text("accuracy");
        if let Some(metres) = form.integer("distance") {
            location.distance = metres as f64;
        }
        if let Some(interval) = form.duration("interval") {
            location.interval = interval;
        }
        location.notification_on_tap_bring_to_front = form.flag("notification");
    }
    if let Some(api_key) = known.api_key_mut() {
        *api_key = form.text("api_key");
    }
    if let Some(scan) = known.scan_mut() {
        // Typed as one comma-separated line, so an empty field means "any
        // device" rather than a list containing one empty string.
        scan.service_uuids = form
            .text("service_uuids")
            .split(',')
            .map(str::trim)
            .filter(|uuid| !uuid.is_empty())
            .map(str::to_owned)
            .collect();
        scan.allow_duplicates = form.flag("allow_duplicates");
    }

    match known.as_mut() {
        KnownDevice::MovesenseDevice { device_type, .. } => {
            *device_type = form.text("device_type");
        }
        KnownDevice::Cams2PolarDevice { name_prefix, .. } => {
            *name_prefix = form.text("name_prefix");
        }
        KnownDevice::CortriumDevice {
            device_type,
            name,
            sampling_rate,
            ..
        } => {
            *device_type = form.text("device_type");
            *name = form.text("name");
            if let Some(rate) = form.integer("sampling_rate") {
                *sampling_rate = rate as u32;
            }
        }
        // Everything else is fully covered by the shared settings above.
        _ => {}
    }

    Applied::Changed
}
