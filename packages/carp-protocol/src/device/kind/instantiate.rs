// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Building a device of a given kind, with the defaults its class ships with.

use super::super::classes::{BluetoothScan, KnownDevice, LocationSettings};
use super::super::{Device, DeviceCore};
use super::DeviceKind;
use crate::duration::Micros;

impl DeviceKind {
    /// Build a device of this kind, with the defaults each class ships with.
    ///
    /// Connected devices are created optional: a study should still deploy
    /// when a participant has no chest strap paired yet.
    pub fn instantiate(self, role_name: String) -> Device {
        let core = DeviceCore::new(role_name, !self.is_primary());
        // One reading a minute, which is the default CAMS uses.
        let location = LocationSettings {
            accuracy: "balanced".to_owned(),
            distance: 10.0,
            interval: Micros::from_seconds(60),
            notification_on_tap_bring_to_front: false,
        };
        // Scan for anything, which is what a study wants before it knows
        // which model the participants were handed.
        let scan = BluetoothScan {
            service_uuids: Vec::new(),
            allow_duplicates: true,
        };

        let device = match self {
            Self::Smartphone => KnownDevice::Smartphone {
                core,
                is_primary_device: true,
            },
            Self::Cams2Smartphone => KnownDevice::Cams2Smartphone {
                core,
                is_primary_device: true,
            },
            Self::WebBrowser => KnownDevice::WebBrowser {
                core,
                is_primary_device: true,
            },
            Self::LocationService => KnownDevice::LocationService { core, location },
            Self::Cams2LocationService => KnownDevice::Cams2LocationService { core, location },
            Self::HealthService => KnownDevice::HealthService { core },
            Self::Cams2HealthService => KnownDevice::Cams2HealthService { core },
            Self::WeatherService => KnownDevice::WeatherService {
                core,
                api_key: String::new(),
            },
            Self::Cams2WeatherService => KnownDevice::Cams2WeatherService {
                core,
                api_key: String::new(),
            },
            Self::AirQualityService => KnownDevice::AirQualityService {
                core,
                api_key: String::new(),
            },
            Self::Cams2AirQualityService => KnownDevice::Cams2AirQualityService {
                core,
                api_key: String::new(),
            },
            Self::PolarDevice => KnownDevice::PolarDevice { core },
            Self::Cams2PolarDevice => KnownDevice::Cams2PolarDevice {
                core,
                scan,
                name_prefix: "Polar".to_owned(),
            },
            Self::MovesenseDevice => KnownDevice::MovesenseDevice {
                core,
                device_type: "UNKNOWN".to_owned(),
            },
            Self::Cams2MovesenseDevice => KnownDevice::Cams2MovesenseDevice { core, scan },
            Self::CortriumDevice => KnownDevice::CortriumDevice {
                core,
                device_type: "C3W".to_owned(),
                name: String::new(),
                sampling_rate: 256,
            },
        };
        Device::Known(Box::new(device))
    }
}
