// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! The device classes, in both namespaces CARP uses.
//!
//! CAMS 2.0 moved the device classes from
//! `dk.cachet.carp.common.application.devices` to `dk.carp.cams.devices`, and
//! it is tempting to treat that as a prefix change. It is not: the CAMS 2.0
//! Bluetooth devices carry different fields. `MovesenseDevice` drops
//! `deviceType` and gains `serviceUuids` and `allowDuplicates`; `PolarDevice`
//! gains those plus `namePrefix`.
//!
//! So each namespace gets its own variants. A rewritten prefix would have
//! produced a document with the wrong fields for the class it claimed to be,
//! which the study app would reject - the sort of error that is easy to write
//! and hard to see.
//!
//! Variants are named for the namespace they belong to: the plain names are
//! the original classes, the `Cams2` ones are the newer namespace.

use serde::{Deserialize, Serialize};

use super::DeviceCore;
use crate::duration::Micros;

/// The device classes this crate models.
///
/// Each variant renames to the fully qualified Kotlin class CARP serialises,
/// which is what the `__type` discriminator carries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "__type", rename_all_fields = "camelCase")]
pub enum KnownDevice {
    // -- primary devices -------------------------------------------------
    #[serde(rename = "dk.cachet.carp.common.application.devices.Smartphone")]
    Smartphone {
        #[serde(flatten)]
        core: DeviceCore,
        #[serde(default = "yes")]
        is_primary_device: bool,
    },

    #[serde(rename = "dk.carp.cams.devices.Smartphone")]
    Cams2Smartphone {
        #[serde(flatten)]
        core: DeviceCore,
        #[serde(default = "yes")]
        is_primary_device: bool,
    },

    #[serde(rename = "dk.cachet.carp.common.application.devices.WebBrowser")]
    WebBrowser {
        #[serde(flatten)]
        core: DeviceCore,
        #[serde(default = "yes")]
        is_primary_device: bool,
    },

    // -- services --------------------------------------------------------
    /// The phone's location provider.
    #[serde(rename = "dk.cachet.carp.common.application.devices.LocationService")]
    LocationService {
        #[serde(flatten)]
        core: DeviceCore,
        #[serde(flatten)]
        location: LocationSettings,
    },

    #[serde(rename = "dk.carp.cams.devices.LocationService")]
    Cams2LocationService {
        #[serde(flatten)]
        core: DeviceCore,
        #[serde(flatten)]
        location: LocationSettings,
    },

    /// The phone's health database (Apple Health / Health Connect).
    #[serde(rename = "dk.cachet.carp.common.application.devices.HealthService")]
    HealthService {
        #[serde(flatten)]
        core: DeviceCore,
    },

    #[serde(rename = "dk.carp.cams.devices.HealthService")]
    Cams2HealthService {
        #[serde(flatten)]
        core: DeviceCore,
    },

    #[serde(rename = "dk.cachet.carp.common.application.devices.WeatherService")]
    WeatherService {
        #[serde(flatten)]
        core: DeviceCore,
        /// OpenWeatherMap key. Stored in the protocol, so it reaches every
        /// participant's phone: only ever put a restricted key here.
        api_key: String,
    },

    #[serde(rename = "dk.carp.cams.devices.WeatherService")]
    Cams2WeatherService {
        #[serde(flatten)]
        core: DeviceCore,
        api_key: String,
    },

    #[serde(rename = "dk.cachet.carp.common.application.devices.AirQualityService")]
    AirQualityService {
        #[serde(flatten)]
        core: DeviceCore,
        /// World Air Quality Index key, with the same caveat as the weather one.
        api_key: String,
    },

    #[serde(rename = "dk.carp.cams.devices.AirQualityService")]
    Cams2AirQualityService {
        #[serde(flatten)]
        core: DeviceCore,
        api_key: String,
    },

    // -- Bluetooth devices ------------------------------------------------
    #[serde(rename = "dk.cachet.carp.common.application.devices.PolarDevice")]
    PolarDevice {
        #[serde(flatten)]
        core: DeviceCore,
    },

    /// The CAMS 2.0 Polar device, which describes how to find the strap
    /// rather than assuming a single one.
    #[serde(rename = "dk.carp.cams.devices.PolarDevice")]
    Cams2PolarDevice {
        #[serde(flatten)]
        core: DeviceCore,
        #[serde(flatten)]
        scan: BluetoothScan,
        /// Only pair with devices whose name starts with this, e.g. `Polar`.
        #[serde(default)]
        name_prefix: String,
    },

    #[serde(rename = "dk.cachet.carp.common.application.devices.MovesenseDevice")]
    MovesenseDevice {
        #[serde(flatten)]
        core: DeviceCore,
        /// `"UNKNOWN"`, `"MD"`, `"HR2"` or another Movesense model code.
        #[serde(default = "unknown")]
        device_type: String,
    },

    /// The CAMS 2.0 Movesense device. It has no model code: the sensor is
    /// identified by its advertised services instead.
    #[serde(rename = "dk.carp.cams.devices.MovesenseDevice")]
    Cams2MovesenseDevice {
        #[serde(flatten)]
        core: DeviceCore,
        #[serde(flatten)]
        scan: BluetoothScan,
    },

    #[serde(rename = "dk.cachet.carp.common.application.devices.CortriumDevice")]
    CortriumDevice {
        #[serde(flatten)]
        core: DeviceCore,
        /// Model code, e.g. `"C3W"`.
        device_type: String,
        /// Serial number of the specific unit, e.g. `"C3W150120"`.
        name: String,
        /// ECG samples per second.
        sampling_rate: u32,
    },
}

/// How a location service samples, shared by both namespaces.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationSettings {
    /// `"reduced"`, `"low"`, `"balanced"`, `"high"`, `"best"` or
    /// `"bestForNavigation"`.
    #[serde(default = "balanced")]
    pub accuracy: String,
    /// Metres of movement before a new reading is taken.
    #[serde(default)]
    pub distance: f64,
    /// Microseconds between readings.
    #[serde(default)]
    pub interval: Micros,
    #[serde(default)]
    pub notification_on_tap_bring_to_front: bool,
}

/// How a CAMS 2.0 Bluetooth device is found.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BluetoothScan {
    /// GATT service UUIDs to scan for. Empty accepts any.
    #[serde(default)]
    pub service_uuids: Vec<String>,
    /// Whether the same device may be reported more than once per scan.
    #[serde(default)]
    pub allow_duplicates: bool,
}

fn yes() -> bool {
    true
}

fn balanced() -> String {
    "balanced".to_owned()
}

fn unknown() -> String {
    "UNKNOWN".to_owned()
}
