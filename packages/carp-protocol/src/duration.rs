// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Durations on the CARP wire.
//!
//! Every duration in a protocol document is a bare integer count of
//! **microseconds**: a daily trigger reads `"period": 86400000000`, and the
//! thirty-day look-back of a health task reads `2592000000000`. Nothing in the
//! JSON says so, which makes a wrong unit a plausible and expensive mistake -
//! a `period` off by a factor of a thousand turns an hourly survey into one
//! every four days.
//!
//! [`Micros`] therefore wraps the integer. It serialises as that integer and
//! nothing else, so the wire format is unchanged, but in Rust the unit is part
//! of the type, and the editor can show and accept `30d` or `1h 30m` instead
//! of asking anyone to count zeroes.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Microseconds per second, minute, hour and day.
const PER_SECOND: i64 = 1_000_000;
const PER_MINUTE: i64 = 60 * PER_SECOND;
const PER_HOUR: i64 = 60 * PER_MINUTE;
const PER_DAY: i64 = 24 * PER_HOUR;

/// A duration in microseconds, the unit CARP uses on the wire.
///
/// Serialises as a plain integer, so `Micros::from_days(1)` writes
/// `86400000000`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Micros(pub i64);

impl Micros {
    pub const ZERO: Self = Self(0);

    pub const fn from_micros(micros: i64) -> Self {
        Self(micros)
    }

    pub const fn from_seconds(seconds: i64) -> Self {
        Self(seconds * PER_SECOND)
    }

    pub const fn from_minutes(minutes: i64) -> Self {
        Self(minutes * PER_MINUTE)
    }

    pub const fn from_hours(hours: i64) -> Self {
        Self(hours * PER_HOUR)
    }

    pub const fn from_days(days: i64) -> Self {
        Self(days * PER_DAY)
    }

    pub const fn micros(self) -> i64 {
        self.0
    }

    /// Whole seconds, rounding towards zero.
    pub const fn as_seconds(self) -> i64 {
        self.0 / PER_SECOND
    }

    /// Parse a human duration such as `30d`, `1h30m`, `500ms` or `90`.
    ///
    /// Accepted units are `us`/`µs`, `ms`, `s`, `m`, `h` and `d`. Several
    /// components may be combined, with or without spaces. A bare number is
    /// read as seconds, which is what someone typing `90` into a "period"
    /// field almost always means.
    ///
    /// Returns `None` for anything it cannot read in full, rather than
    /// guessing at a partial parse.
    pub fn parse(input: &str) -> Option<Self> {
        let text = input.trim();
        if text.is_empty() {
            return None;
        }

        // A bare number is seconds.
        if let Ok(seconds) = text.parse::<i64>() {
            return Some(Self::from_seconds(seconds));
        }

        let mut total: i64 = 0;
        let mut digits = String::new();
        let mut unit = String::new();
        let mut components = 0usize;

        // Walk the string accumulating `<digits><unit>` pairs. A component
        // ends at whitespace or at the digit that begins the next one, which
        // is what lets `1h30m` and `1h 30m` both read.
        for character in text.chars() {
            if character.is_ascii_digit() {
                if !unit.is_empty() {
                    total = total.checked_add(component(&digits, &unit)?)?;
                    components += 1;
                    digits.clear();
                    unit.clear();
                }
                digits.push(character);
            } else if character.is_ascii_alphabetic() || character == 'µ' {
                // A unit has to belong to a number.
                if digits.is_empty() {
                    return None;
                }
                unit.push(character);
            } else if character.is_whitespace() {
                if digits.is_empty() && unit.is_empty() {
                    continue;
                }
                // A number whose unit never arrived, as in `1 h`.
                if unit.is_empty() {
                    return None;
                }
                total = total.checked_add(component(&digits, &unit)?)?;
                components += 1;
                digits.clear();
                unit.clear();
            } else {
                return None;
            }
        }

        // Close the component the string ended on.
        if !digits.is_empty() || !unit.is_empty() {
            if unit.is_empty() {
                return None;
            }
            total = total.checked_add(component(&digits, &unit)?)?;
            components += 1;
        }

        (components > 0).then_some(Self(total))
    }

    /// Render as the shortest exact human form: `30d`, `1h 30m`, `250ms`.
    ///
    /// Exact means no rounding - a value that is not a whole number of the
    /// larger units keeps the smaller ones, so the string always parses back
    /// to the same value. Negative durations are rendered with a leading `-`
    /// for diagnostics; they do not occur in protocols and [`Micros::parse`]
    /// does not accept them back.
    pub fn human(self) -> String {
        if self.0 == 0 {
            return "0s".to_owned();
        }

        let negative = self.0 < 0;
        let mut remaining = self.0.unsigned_abs();
        let mut parts = Vec::new();
        for (unit, size) in [
            ("d", PER_DAY),
            ("h", PER_HOUR),
            ("m", PER_MINUTE),
            ("s", PER_SECOND),
            ("ms", 1_000),
        ] {
            let size = size as u64;
            if remaining >= size {
                parts.push(format!("{}{unit}", remaining / size));
                remaining %= size;
            }
        }
        if remaining > 0 {
            parts.push(format!("{remaining}us"));
        }

        let text = parts.join(" ");
        if negative { format!("-{text}") } else { text }
    }
}

/// One `<digits><unit>` component, in microseconds.
fn component(digits: &str, unit: &str) -> Option<i64> {
    let value: i64 = digits.parse().ok()?;
    let scale = match unit {
        "us" | "µs" => 1,
        "ms" => 1_000,
        "s" => PER_SECOND,
        "m" | "min" => PER_MINUTE,
        "h" => PER_HOUR,
        "d" => PER_DAY,
        _ => return None,
    };
    value.checked_mul(scale)
}

impl fmt::Display for Micros {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.human())
    }
}

#[cfg(test)]
mod tests;
