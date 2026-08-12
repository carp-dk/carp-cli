// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Primitive types shared by the CARP models.
//!
//! The OpenAPI document describes the Kotlin classes rather than their wire
//! format: `UUID` is documented as `{ "stringRepresentation": "..." }` and
//! `Instant` as `{ "epochSeconds": .. }`, while kotlinx serialisation emits
//! plain strings for both. Both encodings are accepted here so the client keeps
//! working whichever one a deployment produces.

use std::fmt;

use chrono::{DateTime, TimeZone, Utc};
use serde::de::{Error as DeError, Unexpected};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A CARP identifier. Kept as a string: it is only ever displayed or passed
/// back to the API.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CarpUuid(String);

impl CarpUuid {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// First segment of the UUID, enough to recognise a row in a narrow table.
    pub fn short(&self) -> &str {
        self.0.split('-').next().unwrap_or(&self.0)
    }
}

impl fmt::Display for CarpUuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for CarpUuid {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Serialize for CarpUuid {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for CarpUuid {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match serde_json::Value::deserialize(deserializer)? {
            serde_json::Value::String(value) => Ok(Self(value)),
            serde_json::Value::Object(map) => map
                .get("stringRepresentation")
                .and_then(serde_json::Value::as_str)
                .map(|value| Self(value.to_owned()))
                .ok_or_else(|| D::Error::custom("UUID object without stringRepresentation")),
            serde_json::Value::Null => Ok(Self::default()),
            other => Err(D::Error::invalid_type(
                Unexpected::Other(&other.to_string()),
                &"a UUID string or object",
            )),
        }
    }
}

/// A point in time as reported by CARP.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CarpInstant(DateTime<Utc>);

impl CarpInstant {
    pub fn as_datetime(&self) -> DateTime<Utc> {
        self.0
    }

    /// `2026-08-11 14:32` in the local time zone.
    pub fn to_local_string(self) -> String {
        self.0
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d %H:%M")
            .to_string()
    }

    /// `2026-08-11` in the local time zone.
    pub fn to_local_date(self) -> String {
        self.0
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d")
            .to_string()
    }
}

impl fmt::Display for CarpInstant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_local_string())
    }
}

impl Serialize for CarpInstant {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_rfc3339())
    }
}

impl<'de> Deserialize<'de> for CarpInstant {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        from_json(&value).ok_or_else(|| D::Error::custom(format!("invalid instant: {value}")))
    }
}

fn from_json(value: &serde_json::Value) -> Option<CarpInstant> {
    match value {
        serde_json::Value::String(text) => parse_text(text),
        serde_json::Value::Number(number) => number.as_i64().and_then(from_epoch),
        serde_json::Value::Object(map) => {
            let seconds = map.get("epochSeconds")?.as_i64()?;
            let nanos = map
                .get("nanosecondsOfSecond")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0) as u32;
            Utc.timestamp_opt(seconds, nanos).single().map(CarpInstant)
        }
        _ => None,
    }
}

fn parse_text(text: &str) -> Option<CarpInstant> {
    if let Ok(parsed) = DateTime::parse_from_rfc3339(text) {
        return Some(CarpInstant(parsed.with_timezone(&Utc)));
    }
    // Kotlin's `LocalDateTime` renders without a zone; assume UTC.
    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(text, "%Y-%m-%dT%H:%M:%S%.f") {
        return Some(CarpInstant(Utc.from_utc_datetime(&naive)));
    }
    text.parse::<i64>().ok().and_then(from_epoch)
}

fn from_epoch(value: i64) -> Option<CarpInstant> {
    // Values this large can only be milliseconds.
    let (seconds, nanos) = if value.abs() >= 100_000_000_000 {
        (
            value / 1_000,
            (value % 1_000).unsigned_abs() as u32 * 1_000_000,
        )
    } else {
        (value, 0)
    };
    Utc.timestamp_opt(seconds, nanos).single().map(CarpInstant)
}

/// Render an optional timestamp for display.
pub fn format_instant(value: Option<CarpInstant>) -> String {
    value.map_or_else(|| "-".to_owned(), CarpInstant::to_local_string)
}

/// Human readable byte count, e.g. `4.2 MB`.
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuid_accepts_both_encodings() {
        let plain: CarpUuid = serde_json::from_str("\"abc-def\"").unwrap();
        let wrapped: CarpUuid =
            serde_json::from_str(r#"{"stringRepresentation":"abc-def"}"#).unwrap();
        assert_eq!(plain, wrapped);
        assert_eq!(plain.short(), "abc");
    }

    #[test]
    fn instant_accepts_both_encodings() {
        let text: CarpInstant = serde_json::from_str("\"2026-08-11T10:00:00Z\"").unwrap();
        let seconds = text.as_datetime().timestamp();
        let object: CarpInstant = serde_json::from_str(&format!(
            r#"{{"epochSeconds":{seconds},"nanosecondsOfSecond":0}}"#
        ))
        .unwrap();
        let millis: CarpInstant = serde_json::from_str(&(seconds * 1000).to_string()).unwrap();
        assert_eq!(text, object);
        assert_eq!(text, millis);
    }

    #[test]
    fn bytes_are_scaled() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1536), "1.5 KB");
    }
}
