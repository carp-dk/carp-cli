// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! [`DeviceKind`]: the device classes the editor can create, and what each one
//! means.
//!
//! This is the *modelled* set: the classes with typed fields and sensible
//! defaults. A protocol may contain devices outside it - they load as
//! [`crate::node::UnknownNode`] and keep working - but only these can be added
//! from the editor, because only these have fields it knows how to ask for.
//!
//! Kinds come in two [`Namespace`]s. A protocol should use one throughout: the
//! study app reads `applicationData.protocolApiLevel` to decide which classes
//! it expects, and mixing them produces a document only half of which it
//! understands.

pub mod instantiate;

/// Which generation of CARP device classes a kind belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Namespace {
    /// `dk.cachet.carp.common.application.devices`, used by CARP core and by
    /// CAMS before 2.0.
    Core,
    /// `dk.carp.cams.devices`, introduced by CAMS 2.0.
    Cams2,
}

impl Namespace {
    pub const ALL: [Self; 2] = [Self::Core, Self::Cams2];

    pub fn label(self) -> &'static str {
        match self {
            Self::Core => "CARP core",
            Self::Cams2 => "CAMS 2.0",
        }
    }

    /// The `protocolApiLevel` a protocol using this namespace declares.
    pub fn api_level(self) -> Option<&'static str> {
        match self {
            Self::Core => None,
            Self::Cams2 => Some("2.0"),
        }
    }

    /// The namespace a protocol's API level implies.
    pub fn for_api_level(api_level: Option<&str>) -> Self {
        match api_level {
            Some("2.0") => Self::Cams2,
            _ => Self::Core,
        }
    }
}

/// A device class that can be added to a protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceKind {
    /// The participant's phone, running CARP Mobile Sensing.
    Smartphone,
    Cams2Smartphone,
    /// A desktop browser, for studies delivered as web tasks.
    WebBrowser,
    LocationService,
    Cams2LocationService,
    HealthService,
    Cams2HealthService,
    WeatherService,
    Cams2WeatherService,
    AirQualityService,
    Cams2AirQualityService,
    PolarDevice,
    Cams2PolarDevice,
    MovesenseDevice,
    Cams2MovesenseDevice,
    CortriumDevice,
}

impl DeviceKind {
    /// Every kind, ordered as the editor's picker shows them: what can carry
    /// a study, then the services, then the wearables, each namespace's
    /// version next to the other's.
    pub const ALL: [Self; 16] = [
        Self::Smartphone,
        Self::Cams2Smartphone,
        Self::WebBrowser,
        Self::LocationService,
        Self::Cams2LocationService,
        Self::HealthService,
        Self::Cams2HealthService,
        Self::WeatherService,
        Self::Cams2WeatherService,
        Self::AirQualityService,
        Self::Cams2AirQualityService,
        Self::PolarDevice,
        Self::Cams2PolarDevice,
        Self::MovesenseDevice,
        Self::Cams2MovesenseDevice,
        Self::CortriumDevice,
    ];

    /// Which generation this class belongs to.
    pub fn namespace(self) -> Namespace {
        match self {
            Self::Cams2Smartphone
            | Self::Cams2LocationService
            | Self::Cams2HealthService
            | Self::Cams2WeatherService
            | Self::Cams2AirQualityService
            | Self::Cams2PolarDevice
            | Self::Cams2MovesenseDevice => Namespace::Cams2,
            _ => Namespace::Core,
        }
    }

    /// The bare class name, without its namespace.
    pub fn class(self) -> &'static str {
        match self {
            Self::Smartphone | Self::Cams2Smartphone => "Smartphone",
            Self::WebBrowser => "WebBrowser",
            Self::LocationService | Self::Cams2LocationService => "LocationService",
            Self::HealthService | Self::Cams2HealthService => "HealthService",
            Self::WeatherService | Self::Cams2WeatherService => "WeatherService",
            Self::AirQualityService | Self::Cams2AirQualityService => "AirQualityService",
            Self::PolarDevice | Self::Cams2PolarDevice => "PolarDevice",
            Self::MovesenseDevice | Self::Cams2MovesenseDevice => "MovesenseDevice",
            Self::CortriumDevice => "CortriumDevice",
        }
    }

    /// Short name shown in lists, with the namespace when it is not the
    /// original one.
    pub fn label(self) -> &'static str {
        match self {
            Self::Smartphone => "Smartphone",
            Self::Cams2Smartphone => "Smartphone (CAMS 2.0)",
            Self::WebBrowser => "WebBrowser",
            Self::LocationService => "LocationService",
            Self::Cams2LocationService => "LocationService (CAMS 2.0)",
            Self::HealthService => "HealthService",
            Self::Cams2HealthService => "HealthService (CAMS 2.0)",
            Self::WeatherService => "WeatherService",
            Self::Cams2WeatherService => "WeatherService (CAMS 2.0)",
            Self::AirQualityService => "AirQualityService",
            Self::Cams2AirQualityService => "AirQualityService (CAMS 2.0)",
            Self::PolarDevice => "PolarDevice",
            Self::Cams2PolarDevice => "PolarDevice (CAMS 2.0)",
            Self::MovesenseDevice => "MovesenseDevice",
            Self::Cams2MovesenseDevice => "MovesenseDevice (CAMS 2.0)",
            Self::CortriumDevice => "CortriumDevice",
        }
    }

    /// One line explaining what the device is, shown beside the picker.
    pub fn description(self) -> &'static str {
        match self {
            Self::Smartphone | Self::Cams2Smartphone => {
                "The participant's phone, running the study app"
            }
            Self::WebBrowser => "A desktop browser running web tasks",
            Self::LocationService | Self::Cams2LocationService => "The phone's location provider",
            Self::HealthService | Self::Cams2HealthService => {
                "Apple Health or Health Connect on the phone"
            }
            Self::WeatherService | Self::Cams2WeatherService => {
                "OpenWeatherMap, sampled at the phone's location"
            }
            Self::AirQualityService | Self::Cams2AirQualityService => {
                "World Air Quality Index, at the phone's location"
            }
            Self::PolarDevice | Self::Cams2PolarDevice => {
                "A Polar chest strap or watch, over Bluetooth"
            }
            Self::MovesenseDevice | Self::Cams2MovesenseDevice => {
                "A Movesense ECG/IMU sensor, over Bluetooth"
            }
            Self::CortriumDevice => "A Cortrium C3+ ECG monitor, over Bluetooth",
        }
    }

    /// The `__type` discriminator this kind serialises as.
    pub fn type_name(self) -> &'static str {
        match self {
            Self::Smartphone => "dk.cachet.carp.common.application.devices.Smartphone",
            Self::Cams2Smartphone => "dk.carp.cams.devices.Smartphone",
            Self::WebBrowser => "dk.cachet.carp.common.application.devices.WebBrowser",
            Self::LocationService => "dk.cachet.carp.common.application.devices.LocationService",
            Self::Cams2LocationService => "dk.carp.cams.devices.LocationService",
            Self::HealthService => "dk.cachet.carp.common.application.devices.HealthService",
            Self::Cams2HealthService => "dk.carp.cams.devices.HealthService",
            Self::WeatherService => "dk.cachet.carp.common.application.devices.WeatherService",
            Self::Cams2WeatherService => "dk.carp.cams.devices.WeatherService",
            Self::AirQualityService => {
                "dk.cachet.carp.common.application.devices.AirQualityService"
            }
            Self::Cams2AirQualityService => "dk.carp.cams.devices.AirQualityService",
            Self::PolarDevice => "dk.cachet.carp.common.application.devices.PolarDevice",
            Self::Cams2PolarDevice => "dk.carp.cams.devices.PolarDevice",
            Self::MovesenseDevice => "dk.cachet.carp.common.application.devices.MovesenseDevice",
            Self::Cams2MovesenseDevice => "dk.carp.cams.devices.MovesenseDevice",
            Self::CortriumDevice => "dk.cachet.carp.common.application.devices.CortriumDevice",
        }
    }

    /// Whether devices of this kind run a study themselves.
    ///
    /// Primary devices go in `primaryDevices` and get a deployment; everything
    /// else goes in `connectedDevices` and needs a
    /// [`crate::control::DeviceConnection`] to a primary device.
    pub fn is_primary(self) -> bool {
        matches!(
            self,
            Self::Smartphone | Self::Cams2Smartphone | Self::WebBrowser
        )
    }

    /// Whether the kind needs an API key, which the editor must then ask for.
    pub fn needs_api_key(self) -> bool {
        matches!(
            self,
            Self::WeatherService
                | Self::Cams2WeatherService
                | Self::AirQualityService
                | Self::Cams2AirQualityService
        )
    }

    /// The role name suggested when adding one, matching the naming the
    /// reference protocols use.
    pub fn default_role_name(self) -> &'static str {
        match self {
            Self::Smartphone | Self::Cams2Smartphone => "Primary Phone",
            Self::WebBrowser => "Web Browser",
            Self::LocationService | Self::Cams2LocationService => "Location Service",
            Self::HealthService | Self::Cams2HealthService => "Health Service",
            Self::WeatherService | Self::Cams2WeatherService => "Weather Service",
            Self::AirQualityService | Self::Cams2AirQualityService => "Air Quality Service",
            Self::PolarDevice | Self::Cams2PolarDevice => "Polar HR Device",
            Self::MovesenseDevice | Self::Cams2MovesenseDevice => "Movesense ECG Device",
            Self::CortriumDevice => "Cortrium ECG Monitor",
        }
    }

    /// Parse a `__type` discriminator back to a kind.
    pub fn from_type_name(type_name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.type_name() == type_name)
    }

    /// The same class in `namespace`, when it exists there.
    ///
    /// `WebBrowser` and `CortriumDevice` have no CAMS 2.0 form, so switching a
    /// protocol's generation leaves them alone.
    pub fn in_namespace(self, namespace: Namespace) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.class() == self.class() && kind.namespace() == namespace)
    }
}

#[cfg(test)]
mod tests;
