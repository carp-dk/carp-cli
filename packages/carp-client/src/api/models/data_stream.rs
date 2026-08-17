// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Data stream models (`data-stream-controller`, `study-deployment-controller`).
//!
//! A *data stream* is one kind of measurement from one device in one
//! deployment — the heart rate from a participant's phone, say. It is the level
//! at which CARP stores what a study collected, so it is the level at which
//! anything reading a study's data has to ask.

use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize};

use crate::api::models::common::{CarpInstant, CarpUuid};

/// Upload volume for one task on one day.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DataPointCount {
    pub date: Option<CarpInstant>,
    pub task: String,
    pub quantity: i64,
}

/// Response of `GET /api/data-stream-service/summary`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DataStreamSummary {
    pub study_id: String,
    pub deployment_id: Option<String>,
    pub participant_id: Option<String>,
    pub scope: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub from: Option<CarpInstant>,
    pub to: Option<CarpInstant>,
    pub data: Vec<DataPointCount>,
}

impl DataStreamSummary {
    pub fn total(&self) -> i64 {
        self.data.iter().map(|point| point.quantity).sum()
    }
}

/// Per-deployment upload counts from `POST /api/deployment-service/statistics`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DeploymentStatistics {
    /// deployment id -> statistic name -> counts
    pub statistics: BTreeMap<String, BTreeMap<String, DeploymentStatistic>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DeploymentStatistic {
    pub count: i32,
    pub uploads: BTreeMap<String, i32>,
}

/// A CARP type name, as `dk.cachet.carp.heartrate`.
///
/// Sent as its two halves because that is what the API takes, but written and
/// read as one dotted string, which is how everything else refers to it — the
/// protocol documents, the study app, the people using them.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NamespacedId {
    pub namespace: String,
    pub name: String,
}

impl NamespacedId {
    pub fn new(namespace: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            name: name.into(),
        }
    }
}

impl fmt::Display for NamespacedId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.namespace.is_empty() {
            f.write_str(&self.name)
        } else {
            write!(f, "{}.{}", self.namespace, self.name)
        }
    }
}

impl FromStr for NamespacedId {
    type Err = String;

    /// Splits at the *last* dot: the namespace is the qualified part and the
    /// name is one segment, so `dk.cachet.carp.heartrate` is
    /// `dk.cachet.carp` + `heartrate`.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.trim();
        if value.is_empty() {
            return Err("a data type cannot be empty".to_owned());
        }
        Ok(match value.rsplit_once('.') {
            Some((namespace, name)) if !namespace.is_empty() && !name.is_empty() => {
                Self::new(namespace, name)
            }
            // No dot at all: an unqualified name. CARP would not send one, but
            // refusing it would be worse than passing it through for the
            // server to reject with a message about the actual type.
            _ => Self::new("", value),
        })
    }
}

/// Identifies one stream: what was measured, by which device, in which
/// deployment. The request body of `getDataStream` and `queryDataStreamByTime`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DataStreamId {
    pub study_deployment_id: CarpUuid,
    pub device_role_name: String,
    pub data_type: NamespacedId,
}

impl DataStreamId {
    pub fn new(
        study_deployment_id: impl Into<String>,
        device_role_name: impl Into<String>,
        data_type: NamespacedId,
    ) -> Self {
        Self {
            study_deployment_id: CarpUuid::new(study_deployment_id),
            device_role_name: device_role_name.into(),
            data_type,
        }
    }
}

/// What `getDataStream` and `queryDataStreamByTime` answer with.
///
/// The OpenAPI document types the payload as a bare object — CARP serialises it
/// with a hand-written `DataStreamBatchSerializer`, which Springdoc cannot see
/// through — so there is no schema here to be written against. Both shapes the
/// serialiser can plausibly produce are therefore accepted, in the same spirit
/// as [`CarpUuid`], and every field not modelled below is preserved in `extra`
/// rather than dropped. `carp data query --raw` prints the response untouched,
/// which is the escape hatch when a deployment sends something unforeseen.
#[derive(Debug, Clone, Default, Serialize)]
pub struct DataStreamBatch {
    pub sequences: Vec<DataStreamSequence>,
}

impl DataStreamBatch {
    pub fn is_empty(&self) -> bool {
        self.sequences.iter().all(|s| s.measurements.is_empty())
    }

    pub fn measurement_count(&self) -> usize {
        self.sequences.iter().map(|s| s.measurements.len()).sum()
    }

    /// One row per measurement, with the stream it came from folded into each.
    ///
    /// A batch nests measurements under the sequence that carried them, which
    /// is how CARP stores them but not how anything analyses them. Tables,
    /// NDJSON and a DataFrame all want the flat form.
    pub fn rows(&self) -> Vec<MeasurementRow> {
        self.sequences
            .iter()
            .flat_map(DataStreamSequence::rows)
            .collect()
    }
}

impl<'de> Deserialize<'de> for DataStreamBatch {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        let sequences = match value {
            // A bare list of sequences.
            serde_json::Value::Array(_) => value,
            // Wrapped, as the documented `{ isEmpty, sequences }` shape.
            serde_json::Value::Object(mut map) => map
                .remove("sequences")
                .unwrap_or(serde_json::Value::Array(Vec::new())),
            serde_json::Value::Null => serde_json::Value::Array(Vec::new()),
            other => {
                return Err(D::Error::custom(format!(
                    "a data stream batch is a list of sequences or an object holding one, not {other}"
                )));
            }
        };

        // `sequences` may itself be wrapped once more, as the Kotlin type is a
        // `Sequence` rather than a list.
        let sequences = match sequences {
            serde_json::Value::Object(mut map) => map
                .remove("sequences")
                .or_else(|| map.remove("elements"))
                .unwrap_or(serde_json::Value::Array(Vec::new())),
            other => other,
        };

        Ok(Self {
            sequences: serde_json::from_value(sequences).map_err(D::Error::custom)?,
        })
    }
}

/// A run of measurements from one stream, numbered from `first_sequence_id`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DataStreamSequence {
    pub data_stream: DataStreamId,
    pub first_sequence_id: i64,
    pub measurements: Vec<Measurement>,
    /// Which of the protocol's triggers caused this run.
    pub trigger_ids: Vec<i64>,
    /// Anything the server sent that is not modelled above — a `syncPoint`,
    /// or a field added by a newer CARP. Kept so it reaches `--format json`.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl DataStreamSequence {
    /// This sequence's measurements, each carrying its stream and its number.
    pub fn rows(&self) -> impl Iterator<Item = MeasurementRow> + '_ {
        self.measurements
            .iter()
            .enumerate()
            .map(move |(offset, measurement)| MeasurementRow {
                deployment_id: self.data_stream.study_deployment_id.to_string(),
                device_role_name: self.data_stream.device_role_name.clone(),
                data_type: self.data_stream.data_type.to_string(),
                sequence_id: self.first_sequence_id + offset as i64,
                sensor_start_time: measurement.sensor_start_time,
                start: measurement.start(),
                sensor_end_time: measurement.sensor_end_time,
                end: measurement.end(),
                trigger_ids: self.trigger_ids.clone(),
                data: measurement.data.clone(),
            })
    }
}

/// One measurement, as the device recorded it.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Measurement {
    /// Microseconds since the Unix epoch, UTC.
    pub sensor_start_time: i64,
    /// Absent for a measurement taken at an instant rather than over a period.
    pub sensor_end_time: Option<i64>,
    /// The reading itself, `__type`-tagged. Deliberately untyped: CARP has a
    /// class per measure and a study may carry one this build has never heard
    /// of. Anything reading a specific type knows what to expect of it.
    pub data: serde_json::Value,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl Measurement {
    pub fn start(&self) -> Option<DateTime<Utc>> {
        DateTime::from_timestamp_micros(self.sensor_start_time)
    }

    pub fn end(&self) -> Option<DateTime<Utc>> {
        self.sensor_end_time
            .and_then(DateTime::from_timestamp_micros)
    }
}

/// One measurement with its stream folded in, for output that has rows.
///
/// Both times are given twice on purpose: `sensor_start_time` is exactly what
/// the server said, and `start` is that made readable. A CSV wants the second,
/// anything comparing against CARP itself wants the first, and deriving one
/// from the other in every consumer would be worse than carrying both.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeasurementRow {
    pub deployment_id: String,
    pub device_role_name: String,
    pub data_type: String,
    pub sequence_id: i64,
    pub sensor_start_time: i64,
    pub start: Option<DateTime<Utc>>,
    pub sensor_end_time: Option<i64>,
    pub end: Option<DateTime<Utc>>,
    pub trigger_ids: Vec<i64>,
    pub data: serde_json::Value,
}

#[cfg(test)]
mod tests;
