// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Devices: the things a protocol collects data from.
//!
//! CARP models more than hardware here. A phone is a device, but so is the
//! weather service and the phone's own health database: anything that is a
//! source of data and can be addressed by a *role name* is a device. Role
//! names are the identifiers the whole document hangs off - triggers name the
//! device they fire on, task controls name the device a task runs on - so they
//! must be unique across primary and connected devices alike.
//!
//! Devices split into two lists on the protocol:
//!
//! - **primary** devices run the study themselves, and are what CAWS creates a
//!   deployment for: a `Smartphone`, or a `WebBrowser`
//! - **connected** devices are reached through a primary device, and appear in
//!   [`crate::control::DeviceConnection`] saying which one
//!
//! Which list a kind belongs in is a property of the kind, not a free choice;
//! [`DeviceKind::is_primary`] states it.
//!
//! # Two namespaces
//!
//! CAMS 2.0 introduced a second set of device classes under
//! `dk.carp.cams.devices`, and they are not merely renamed - see [`classes`].
//! Both sets are modelled, and [`DeviceKind::namespace`] says which a kind
//! belongs to.

pub mod access;
pub mod classes;
pub mod kind;
pub mod sampling;

use serde::{Deserialize, Serialize};

pub use classes::{BluetoothScan, KnownDevice, LocationSettings};
pub use kind::{DeviceKind, Namespace};
pub use sampling::{KnownSamplingConfiguration, SamplingConfiguration};

use crate::node::UnknownNode;
use std::collections::BTreeMap;

/// A device taking part in a protocol.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Device {
    Known(Box<KnownDevice>),
    /// A device class this version does not model. See [`crate::node`].
    Unknown(UnknownNode),
}

/// Fields every modelled device carries.
///
/// `defaultSamplingConfiguration` maps a measure type to the configuration
/// used whenever a task on this device measures it without overriding.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCore {
    pub role_name: String,
    /// An optional device may be absent at deployment without blocking it.
    /// Primary devices are generally required, connected ones optional.
    ///
    /// `Option` rather than `bool` because the field is genuinely absent from
    /// some documents - the ICAT study's `WebBrowser` omits it - and writing
    /// back an `isOptional: false` that was never there would change the
    /// document. CARP reads an absent flag as `false`, which is what
    /// [`Device::is_optional`] reports.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_optional: Option<bool>,
    #[serde(default)]
    pub default_sampling_configuration: BTreeMap<String, SamplingConfiguration>,
}

impl DeviceCore {
    pub fn new(role_name: impl Into<String>, is_optional: bool) -> Self {
        Self {
            role_name: role_name.into(),
            is_optional: Some(is_optional),
            default_sampling_configuration: BTreeMap::new(),
        }
    }
}

impl Device {
    /// Build a device of `kind` with this role name.
    ///
    /// Optionality follows the kind's convention: primary devices are
    /// required, connected ones optional, which is what every reference
    /// protocol does.
    pub fn new(kind: DeviceKind, role_name: impl Into<String>) -> Self {
        kind.instantiate(role_name.into())
    }

    /// The identifier the rest of the document refers to this device by.
    pub fn role_name(&self) -> &str {
        match self {
            Self::Known(device) => &device.core().role_name,
            Self::Unknown(node) => node.role_name().unwrap_or_default(),
        }
    }

    /// Rename the device. The caller is responsible for updating references;
    /// [`crate::builder`] does it as one operation.
    pub fn set_role_name(&mut self, role_name: impl Into<String>) {
        let role_name = role_name.into();
        match self {
            Self::Known(device) => device.core_mut().role_name = role_name,
            Self::Unknown(node) => {
                node.fields
                    .insert("roleName".to_owned(), serde_json::Value::String(role_name));
            }
        }
    }

    /// Which kind this is, when it is one this version models.
    pub fn kind(&self) -> Option<DeviceKind> {
        match self {
            Self::Known(device) => Some(device.kind()),
            Self::Unknown(_) => None,
        }
    }

    /// The class name to show in a list.
    pub fn type_label(&self) -> &str {
        match self {
            Self::Known(device) => device.kind().label(),
            Self::Unknown(node) => node.short_type(),
        }
    }

    /// Whether the device runs the study itself, and so belongs in
    /// [`crate::StudyProtocol::primary_devices`].
    ///
    /// Unmodelled devices report their own `isPrimaryDevice` flag, defaulting
    /// to connected, which is the commoner case.
    pub fn is_primary(&self) -> bool {
        match self {
            Self::Known(device) => device.kind().is_primary(),
            Self::Unknown(node) => node
                .field("isPrimaryDevice")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
        }
    }

    /// Whether the device may be missing at deployment. An absent flag counts
    /// as required, which is how CARP reads it.
    pub fn is_optional(&self) -> bool {
        match self {
            Self::Known(device) => device.core().is_optional.unwrap_or(false),
            Self::Unknown(node) => node
                .field("isOptional")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
        }
    }

    /// The default sampling configurations, when the device is modelled.
    pub fn sampling(&self) -> Option<&BTreeMap<String, SamplingConfiguration>> {
        match self {
            Self::Known(device) => Some(&device.core().default_sampling_configuration),
            Self::Unknown(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A phone reads back as a primary device with its flag intact.
    #[test]
    fn a_smartphone_round_trips() {
        let original = serde_json::json!({
            "__type": "dk.cachet.carp.common.application.devices.Smartphone",
            "roleName": "Primary Phone",
            "isOptional": false,
            "defaultSamplingConfiguration": {},
            "isPrimaryDevice": true
        });

        let device: Device = serde_json::from_value(original.clone()).unwrap();
        assert_eq!(device.role_name(), "Primary Phone");
        assert!(device.is_primary());
        assert_eq!(device.type_label(), "Smartphone");
        assert_eq!(serde_json::to_value(&device).unwrap(), original);
    }

    /// The two namespaces are distinct types and must not be silently
    /// unified: writing a CAMS 2.0 protocol back under the old name would
    /// change what the study app parses.
    #[test]
    fn the_two_namespaces_stay_apart() {
        let v2 = serde_json::json!({
            "__type": "dk.carp.cams.devices.Smartphone",
            "roleName": "Neuropathy Tracker",
            "isOptional": false,
            "defaultSamplingConfiguration": {},
            "isPrimaryDevice": true
        });

        let device: Device = serde_json::from_value(v2.clone()).unwrap();
        assert_eq!(device.kind(), Some(DeviceKind::Cams2Smartphone));
        assert!(device.is_primary());
        assert_eq!(serde_json::to_value(&device).unwrap(), v2);
    }

    /// Services carry their own settings alongside the shared core fields,
    /// and `flatten` must not swallow or reorder them.
    #[test]
    fn a_location_service_keeps_its_settings() {
        let original = serde_json::json!({
            "__type": "dk.cachet.carp.common.application.devices.LocationService",
            "roleName": "Location Service",
            "isOptional": true,
            "defaultSamplingConfiguration": {},
            "accuracy": "balanced",
            "distance": 10.0,
            "interval": 60000000i64,
            "notificationOnTapBringToFront": false
        });

        let device: Device = serde_json::from_value(original.clone()).unwrap();
        assert!(
            !device.is_primary(),
            "a location service is reached through a phone"
        );
        assert!(device.is_optional());
        assert_eq!(serde_json::to_value(&device).unwrap(), original);
    }

    /// The CAMS 2.0 Bluetooth devices have their own fields, which is why
    /// they are separate classes rather than a renamed prefix.
    #[test]
    fn the_cams2_bluetooth_devices_keep_their_own_fields() {
        let polar = serde_json::json!({
            "__type": "dk.carp.cams.devices.PolarDevice",
            "roleName": "Polar HR Sensor",
            "isOptional": true,
            "defaultSamplingConfiguration": {},
            "serviceUuids": [],
            "namePrefix": "Polar",
            "allowDuplicates": true
        });
        let device: Device = serde_json::from_value(polar.clone()).unwrap();
        assert_eq!(device.kind(), Some(DeviceKind::Cams2PolarDevice));
        assert_eq!(serde_json::to_value(&device).unwrap(), polar);

        // The CAMS 2.0 Movesense has no `deviceType`, where the older one
        // requires it.
        let movesense = serde_json::json!({
            "__type": "dk.carp.cams.devices.MovesenseDevice",
            "roleName": "Movesense ECG Device",
            "isOptional": true,
            "defaultSamplingConfiguration": {},
            "serviceUuids": [],
            "allowDuplicates": true
        });
        let device: Device = serde_json::from_value(movesense.clone()).unwrap();
        assert_eq!(device.kind(), Some(DeviceKind::Cams2MovesenseDevice));
        assert_eq!(serde_json::to_value(&device).unwrap(), movesense);
    }

    #[test]
    fn renaming_updates_known_and_unknown_devices_alike() {
        let mut known = Device::new(DeviceKind::Smartphone, "Old");
        known.set_role_name("New");
        assert_eq!(known.role_name(), "New");

        let mut unknown = Device::Unknown(UnknownNode {
            type_name: "dk.cachet.carp.common.application.devices.Future".to_owned(),
            fields: serde_json::Map::new(),
        });
        unknown.set_role_name("New");
        assert_eq!(unknown.role_name(), "New");
    }
}
