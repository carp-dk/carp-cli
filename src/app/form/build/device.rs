// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Building the form for a device.
//!
//! Settings are gathered through the accessors on
//! [`carp_protocol::device::KnownDevice`] rather than by matching every
//! variant, because the same setting exists under both CARP namespaces and
//! the form should not care which generation it is looking at. Only the
//! genuinely class-specific fields are matched out.

use carp_protocol::device::{Device, KnownDevice};

use crate::app::form::{Field, FieldValue, Form, Subject, Vocabulary};

/// A device, showing only the settings its class actually has.
///
/// An unmodelled device gets the two common rows and nothing else: its own
/// fields are preserved on save but cannot be shown, because this build does
/// not know what they mean. [`carp_protocol::validate()`] says so explicitly.
pub fn device(device: &Device) -> Form {
    let mut fields = vec![
        // Typed rather than picked: a role name is chosen, not looked up.
        // The conventional names other studies use are listed in the Catalog
        // tab instead of being pushed into an overlay here.
        Field::new(
            "role_name",
            "Role name",
            FieldValue::Text(device.role_name().to_owned()),
        )
        .with_help("How triggers and task controls refer to this device"),
        Field::new(
            "is_optional",
            "Optional",
            FieldValue::Toggle(device.is_optional()),
        )
        .with_help("An optional device may be missing when the study deploys"),
    ];

    if let Device::Known(known) = device {
        fields.extend(class_fields(known));
    }

    Form::new(Subject::Device(device.role_name().to_owned()), fields)
}

/// The rows particular to one device class.
fn class_fields(device: &KnownDevice) -> Vec<Field> {
    let mut fields = Vec::new();

    if let Some(location) = device.location() {
        fields.extend([
            Field::new(
                "accuracy",
                "Accuracy",
                FieldValue::Catalog {
                    vocabulary: Vocabulary::LocationAccuracies,
                    value: location.accuracy.clone(),
                },
            )
            .with_help("Higher accuracy costs more battery"),
            Field::new(
                "distance",
                "Distance (m)",
                FieldValue::Integer {
                    // The wire type is a float, but a sub-metre threshold is
                    // meaningless to a phone's location provider, so the
                    // editor works in whole metres.
                    value: location.distance as i64,
                    min: 0,
                    max: 100_000,
                },
            )
            .with_help("Metres of movement before a new reading is taken"),
            Field::new(
                "interval",
                "Interval",
                FieldValue::Duration(location.interval),
            )
            .with_help("Time between readings, e.g. 60s"),
            Field::new(
                "notification",
                "Bring app to front on tap",
                FieldValue::Toggle(location.notification_on_tap_bring_to_front),
            ),
        ]);
    }

    if let Some(api_key) = device.api_key() {
        fields.push(
            Field::new("api_key", "API key", FieldValue::Text(api_key.to_owned())).with_help(
                "Stored in the protocol, so it reaches every phone: use a restricted key",
            ),
        );
    }

    if let Some(scan) = device.scan() {
        fields.extend([
            Field::new(
                "service_uuids",
                "Service UUIDs",
                FieldValue::Text(scan.service_uuids.join(", ")),
            )
            .with_help("Comma-separated GATT service UUIDs; empty accepts any device"),
            Field::new(
                "allow_duplicates",
                "Allow duplicates",
                FieldValue::Toggle(scan.allow_duplicates),
            )
            .with_help("Whether one device may be reported more than once per scan"),
        ]);
    }

    match device {
        KnownDevice::MovesenseDevice { device_type, .. } => fields.push(
            Field::new(
                "device_type",
                "Model",
                FieldValue::Text(device_type.clone()),
            )
            .with_help("Movesense model code, or UNKNOWN to accept any"),
        ),

        KnownDevice::Cams2PolarDevice { name_prefix, .. } => fields.push(
            Field::new(
                "name_prefix",
                "Name starts with",
                FieldValue::Text(name_prefix.clone()),
            )
            .with_help("Only pair with straps whose advertised name begins with this"),
        ),

        KnownDevice::CortriumDevice {
            device_type,
            name,
            sampling_rate,
            ..
        } => fields.extend([
            Field::new(
                "device_type",
                "Model",
                FieldValue::Text(device_type.clone()),
            ),
            Field::new("name", "Serial number", FieldValue::Text(name.clone()))
                .with_help("Serial of the specific unit, e.g. C3W150120"),
            Field::new(
                "sampling_rate",
                "Sampling rate (Hz)",
                FieldValue::Integer {
                    value: i64::from(*sampling_rate),
                    min: 1,
                    max: 4096,
                },
            ),
        ]),

        // Everything else is fully covered by the shared settings above.
        _ => {}
    }

    fields
}
