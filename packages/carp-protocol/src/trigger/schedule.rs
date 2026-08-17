// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Time-of-day and recurrence values used by the scheduled triggers.

use serde::{Deserialize, Serialize};

use crate::duration::Micros;

/// A wall-clock time in the participant's own time zone.
///
/// A survey scheduled for 20:00 arrives at 20:00 wherever the participant is,
/// which is what a diary study wants and what a UTC instant would get wrong.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeOfDay {
    pub hour: u8,
    pub minute: u8,
    #[serde(default)]
    pub second: u8,
}

impl TimeOfDay {
    pub const fn new(hour: u8, minute: u8) -> Self {
        Self {
            hour,
            minute,
            second: 0,
        }
    }

    /// `20:00`, or `20:00:30` when the seconds matter.
    pub fn label(self) -> String {
        if self.second == 0 {
            format!("{:02}:{:02}", self.hour, self.minute)
        } else {
            format!("{:02}:{:02}:{:02}", self.hour, self.minute, self.second)
        }
    }

    /// Read `HH:MM` or `HH:MM:SS`, rejecting out-of-range values.
    pub fn parse(input: &str) -> Option<Self> {
        let mut parts = input.trim().split(':');
        let hour: u8 = parts.next()?.trim().parse().ok()?;
        let minute: u8 = parts.next()?.trim().parse().ok()?;
        let second: u8 = match parts.next() {
            Some(text) => text.trim().parse().ok()?,
            None => 0,
        };
        // A trailing fourth component means the input was not a time.
        if parts.next().is_some() || hour > 23 || minute > 59 || second > 59 {
            return None;
        }
        Some(Self {
            hour,
            minute,
            second,
        })
    }
}

/// How often a [`super::KnownTrigger::RecurrentScheduled`] repeats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Recurrence {
    Daily,
    Weekly,
    Biweekly,
    Monthly,
}

impl Recurrence {
    pub const ALL: [Self; 4] = [Self::Daily, Self::Weekly, Self::Biweekly, Self::Monthly];

    /// The `type` string CARP writes.
    pub fn wire_name(self) -> &'static str {
        match self {
            Self::Daily => "daily",
            Self::Weekly => "weekly",
            Self::Biweekly => "biweekly",
            Self::Monthly => "monthly",
        }
    }

    pub fn parse(wire_name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|recurrence| recurrence.wire_name() == wire_name)
    }

    /// The `period` that goes with this recurrence.
    ///
    /// CARP stores the period alongside the recurrence type even though one
    /// implies the other, and CAMS reads the period, so the two must agree.
    /// A month is taken as 30 days, matching CAMS' own `RecurrentScheduledTrigger`.
    pub fn period(self) -> Micros {
        match self {
            Self::Daily => Micros::from_days(1),
            Self::Weekly => Micros::from_days(7),
            Self::Biweekly => Micros::from_days(14),
            Self::Monthly => Micros::from_days(30),
        }
    }

    /// Whether a day of the week has to be given as well.
    pub fn needs_day_of_week(self) -> bool {
        matches!(self, Self::Weekly | Self::Biweekly)
    }
}

/// Day of the week as CARP numbers it: Monday is 1, Sunday is 7 (ISO-8601).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DayOfWeek(pub u8);

impl DayOfWeek {
    pub const MONDAY: Self = Self(1);
    pub const SUNDAY: Self = Self(7);

    pub const ALL: [Self; 7] = [
        Self(1),
        Self(2),
        Self(3),
        Self(4),
        Self(5),
        Self(6),
        Self(7),
    ];

    pub fn label(self) -> &'static str {
        match self.0 {
            1 => "Monday",
            2 => "Tuesday",
            3 => "Wednesday",
            4 => "Thursday",
            5 => "Friday",
            6 => "Saturday",
            7 => "Sunday",
            _ => "unknown day",
        }
    }

    pub fn is_valid(self) -> bool {
        (1..=7).contains(&self.0)
    }
}

#[cfg(test)]
mod tests;
