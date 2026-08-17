// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! [`Measure`]: one thing a task collects.
//!
//! A measure names a *data stream type* - `dk.cachet.carp.location`,
//! `dk.cachet.carp.survey` - and optionally overrides how it is sampled. The
//! set of valid type names is not fixed by CARP core: each sampling package a
//! study app links in contributes its own. That is why they are plain strings
//! here and why the valid values are discovered by `carp-catalog` from the
//! upstream configurations rather than being enumerated in this crate.

use serde::{Deserialize, Serialize};

use crate::device::SamplingConfiguration;
use crate::node::{UnknownNode, short_type};

/// One data stream a task collects.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Measure {
    Known(KnownMeasure),
    /// A measure class this version does not model. See [`crate::node`].
    Unknown(UnknownNode),
}

/// The measure classes this crate models.
///
/// CARP defines exactly one today; the enum exists so a second does not force
/// a breaking change.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "__type")]
pub enum KnownMeasure {
    /// Collect a data stream for the whole time the task runs.
    #[serde(
        rename = "dk.cachet.carp.common.application.tasks.Measure.DataStream",
        rename_all = "camelCase"
    )]
    DataStream {
        /// The data stream type, e.g. `dk.cachet.carp.stepcount`.
        r#type: String,
        /// Sampling settings for this measure alone, overriding the device's
        /// default for the type.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        override_sampling_configuration: Option<SamplingConfiguration>,
    },
}

impl Measure {
    /// A measure of `data_type`, sampled however its device says.
    pub fn data_stream(data_type: impl Into<String>) -> Self {
        Self::Known(KnownMeasure::DataStream {
            r#type: data_type.into(),
            override_sampling_configuration: None,
        })
    }

    /// A measure of `data_type` with its own sampling configuration.
    pub fn with_sampling(data_type: impl Into<String>, sampling: SamplingConfiguration) -> Self {
        Self::Known(KnownMeasure::DataStream {
            r#type: data_type.into(),
            override_sampling_configuration: Some(sampling),
        })
    }

    /// The data stream type, or the class name for an unmodelled measure.
    pub fn data_type(&self) -> &str {
        match self {
            Self::Known(KnownMeasure::DataStream { r#type, .. }) => r#type,
            Self::Unknown(node) => node
                .field("type")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_else(|| node.short_type()),
        }
    }

    /// The readable tail of the data type: `stepcount` for
    /// `dk.cachet.carp.stepcount`.
    pub fn short_name(&self) -> &str {
        short_type(self.data_type())
    }

    /// This measure's own sampling configuration, if it has one.
    pub fn sampling(&self) -> Option<&SamplingConfiguration> {
        match self {
            Self::Known(KnownMeasure::DataStream {
                override_sampling_configuration,
                ..
            }) => override_sampling_configuration.as_ref(),
            Self::Unknown(_) => None,
        }
    }

    /// Replace this measure's sampling configuration.
    pub fn set_sampling(&mut self, sampling: Option<SamplingConfiguration>) {
        if let Self::Known(KnownMeasure::DataStream {
            override_sampling_configuration,
            ..
        }) = self
        {
            *override_sampling_configuration = sampling;
        }
    }
}

#[cfg(test)]
mod tests;
