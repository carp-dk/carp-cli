// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Reading and writing the fields device variants share across namespaces.
//!
//! The accessors here are why the editor does not have to know whether it is
//! looking at a CARP core device or a CAMS 2.0 one: the same setting is
//! reachable either way.

use super::classes::{BluetoothScan, KnownDevice, LocationSettings};
use super::DeviceCore;

impl KnownDevice {
    /// The fields shared by every device class.
    pub fn core(&self) -> &DeviceCore {
        match self {
            Self::Smartphone { core, .. }
            | Self::Cams2Smartphone { core, .. }
            | Self::WebBrowser { core, .. }
            | Self::LocationService { core, .. }
            | Self::Cams2LocationService { core, .. }
            | Self::HealthService { core }
            | Self::Cams2HealthService { core }
            | Self::WeatherService { core, .. }
            | Self::Cams2WeatherService { core, .. }
            | Self::AirQualityService { core, .. }
            | Self::Cams2AirQualityService { core, .. }
            | Self::PolarDevice { core }
            | Self::Cams2PolarDevice { core, .. }
            | Self::MovesenseDevice { core, .. }
            | Self::Cams2MovesenseDevice { core, .. }
            | Self::CortriumDevice { core, .. } => core,
        }
    }

    pub fn core_mut(&mut self) -> &mut DeviceCore {
        match self {
            Self::Smartphone { core, .. }
            | Self::Cams2Smartphone { core, .. }
            | Self::WebBrowser { core, .. }
            | Self::LocationService { core, .. }
            | Self::Cams2LocationService { core, .. }
            | Self::HealthService { core }
            | Self::Cams2HealthService { core }
            | Self::WeatherService { core, .. }
            | Self::Cams2WeatherService { core, .. }
            | Self::AirQualityService { core, .. }
            | Self::Cams2AirQualityService { core, .. }
            | Self::PolarDevice { core }
            | Self::Cams2PolarDevice { core, .. }
            | Self::MovesenseDevice { core, .. }
            | Self::Cams2MovesenseDevice { core, .. }
            | Self::CortriumDevice { core, .. } => core,
        }
    }

    /// The location settings, for the two classes that have them.
    ///
    /// Accessors rather than a match at every call site: the same setting
    /// exists under both namespaces, and code that edits it should not have
    /// to care which generation it is looking at.
    pub fn location(&self) -> Option<&LocationSettings> {
        match self {
            Self::LocationService { location, .. } | Self::Cams2LocationService { location, .. } => {
                Some(location)
            }
            _ => None,
        }
    }

    pub fn location_mut(&mut self) -> Option<&mut LocationSettings> {
        match self {
            Self::LocationService { location, .. } | Self::Cams2LocationService { location, .. } => {
                Some(location)
            }
            _ => None,
        }
    }

    /// The API key, for the services that need one.
    pub fn api_key(&self) -> Option<&str> {
        match self {
            Self::WeatherService { api_key, .. }
            | Self::Cams2WeatherService { api_key, .. }
            | Self::AirQualityService { api_key, .. }
            | Self::Cams2AirQualityService { api_key, .. } => Some(api_key),
            _ => None,
        }
    }

    pub fn api_key_mut(&mut self) -> Option<&mut String> {
        match self {
            Self::WeatherService { api_key, .. }
            | Self::Cams2WeatherService { api_key, .. }
            | Self::AirQualityService { api_key, .. }
            | Self::Cams2AirQualityService { api_key, .. } => Some(api_key),
            _ => None,
        }
    }

    /// How a CAMS 2.0 Bluetooth device is discovered.
    pub fn scan(&self) -> Option<&BluetoothScan> {
        match self {
            Self::Cams2PolarDevice { scan, .. } | Self::Cams2MovesenseDevice { scan, .. } => {
                Some(scan)
            }
            _ => None,
        }
    }

    pub fn scan_mut(&mut self) -> Option<&mut BluetoothScan> {
        match self {
            Self::Cams2PolarDevice { scan, .. } | Self::Cams2MovesenseDevice { scan, .. } => {
                Some(scan)
            }
            _ => None,
        }
    }

    pub fn kind(&self) -> super::DeviceKind {
        use super::DeviceKind as K;
        match self {
            Self::Smartphone { .. } => K::Smartphone,
            Self::Cams2Smartphone { .. } => K::Cams2Smartphone,
            Self::WebBrowser { .. } => K::WebBrowser,
            Self::LocationService { .. } => K::LocationService,
            Self::Cams2LocationService { .. } => K::Cams2LocationService,
            Self::HealthService { .. } => K::HealthService,
            Self::Cams2HealthService { .. } => K::Cams2HealthService,
            Self::WeatherService { .. } => K::WeatherService,
            Self::Cams2WeatherService { .. } => K::Cams2WeatherService,
            Self::AirQualityService { .. } => K::AirQualityService,
            Self::Cams2AirQualityService { .. } => K::Cams2AirQualityService,
            Self::PolarDevice { .. } => K::PolarDevice,
            Self::Cams2PolarDevice { .. } => K::Cams2PolarDevice,
            Self::MovesenseDevice { .. } => K::MovesenseDevice,
            Self::Cams2MovesenseDevice { .. } => K::Cams2MovesenseDevice,
            Self::CortriumDevice { .. } => K::CortriumDevice,
        }
    }
}
