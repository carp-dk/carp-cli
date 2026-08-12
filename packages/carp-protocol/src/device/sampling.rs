// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Sampling configurations: *how* a measure is taken, as opposed to *what*.
//!
//! A configuration reaches a measure by one of two routes:
//!
//! - on a device, in `defaultSamplingConfiguration`, keyed by measure type -
//!   it then applies to every task on that device measuring that type
//! - on a single [`crate::task::Measure`], in `overrideSamplingConfiguration`,
//!   which wins for that measure alone
//!
//! Only the types the reference protocols actually configure are modelled;
//! anything else round-trips through [`crate::node::UnknownNode`].

use serde::{Deserialize, Serialize};

use crate::duration::Micros;
use crate::node::UnknownNode;

/// How one measure type is sampled.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SamplingConfiguration {
    Known(KnownSamplingConfiguration),
    /// A configuration class this version does not model. See [`crate::node`].
    Unknown(UnknownNode),
}

/// The sampling configuration classes this crate models.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "__type")]
pub enum KnownSamplingConfiguration {
    /// Which health metrics to read, and over what window.
    ///
    /// The window is relative to the moment of sampling: `past` reaches
    /// backwards, `future` forwards, which matters for metrics like sleep that
    /// the phone writes after the fact.
    #[serde(
        rename = "dk.cachet.carp.common.application.sampling.HealthSamplingConfiguration",
        rename_all = "camelCase"
    )]
    Health {
        /// How far back to read, in microseconds.
        past: Micros,
        /// How far forward to read, in microseconds.
        future: Micros,
        /// Health metric names, e.g. `STEPS`, `SLEEP_SESSION`. The valid set
        /// comes from the `health` Flutter package and is surfaced by
        /// `carp-catalog` rather than fixed here.
        health_data_types: Vec<String>,
    },

    /// Take a single location fix rather than a stream of them.
    #[serde(
        rename = "dk.cachet.carp.common.application.sampling.LocationSamplingConfiguration",
        rename_all = "camelCase"
    )]
    Location { once: bool },

    /// Sample in bursts: collect for `duration`, every `interval`.
    ///
    /// Used for measures that would flatten a battery if left running, such
    /// as the accelerometer.
    #[serde(
        rename = "dk.cachet.carp.common.application.sampling.PeriodicSamplingConfiguration",
        rename_all = "camelCase"
    )]
    Periodic {
        /// Microseconds between the start of one burst and the next.
        interval: Micros,
        /// Microseconds each burst lasts.
        duration: Micros,
    },

    /// The same, for Bluetooth scans, with what to scan for.
    #[serde(
        rename = "dk.cachet.carp.common.application.sampling.BluetoothScanPeriodicSamplingConfiguration",
        rename_all = "camelCase"
    )]
    BluetoothScanPeriodic {
        interval: Micros,
        duration: Micros,
        /// GATT service UUIDs to scan for. Empty accepts any.
        #[serde(default)]
        with_services: Vec<String>,
        /// Specific device ids to look for. Empty accepts any.
        #[serde(default)]
        with_remote_ids: Vec<String>,
    },
}

impl SamplingConfiguration {
    /// A health configuration reading `types` over the last 30 days and the
    /// next day, which is the window the reference protocols use.
    pub fn health(types: Vec<String>) -> Self {
        Self::Known(KnownSamplingConfiguration::Health {
            past: Micros::from_days(30),
            future: Micros::from_days(1),
            health_data_types: types,
        })
    }

    /// A one-shot location fix.
    pub fn location_once() -> Self {
        Self::Known(KnownSamplingConfiguration::Location { once: true })
    }

    /// A one-line summary for the editor.
    pub fn label(&self) -> String {
        match self {
            Self::Known(KnownSamplingConfiguration::Health {
                past,
                health_data_types,
                ..
            }) => format!("{} health types, {} back", health_data_types.len(), past.human()),
            Self::Known(KnownSamplingConfiguration::Location { once }) => {
                if *once { "single fix".to_owned() } else { "continuous".to_owned() }
            }
            Self::Known(KnownSamplingConfiguration::Periodic { interval, duration }) => {
                format!("{} every {}", duration.human(), interval.human())
            }
            Self::Known(KnownSamplingConfiguration::BluetoothScanPeriodic {
                interval,
                duration,
                with_services,
                ..
            }) => {
                let scope = if with_services.is_empty() {
                    "any service".to_owned()
                } else {
                    format!("{} service(s)", with_services.len())
                };
                format!("scan {} every {}, {scope}", duration.human(), interval.human())
            }
            Self::Unknown(node) => node.short_type().to_owned(),
        }
    }

    /// The health metric names, when this is a health configuration.
    pub fn health_data_types(&self) -> Option<&[String]> {
        match self {
            Self::Known(KnownSamplingConfiguration::Health {
                health_data_types, ..
            }) => Some(health_data_types),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests;
