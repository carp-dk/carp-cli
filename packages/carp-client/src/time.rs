// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Points in time, as someone would type one.
//!
//! CARP takes ISO-8601 instants, which nobody wants to write by hand for
//! "the last fortnight". Three forms are accepted and all resolve to the same
//! thing:
//!
//! - an age: `7d`, `36h`, `90m` — that long before now
//! - a date: `2026-08-01` — midnight UTC on that day
//! - an instant: `2026-08-01T09:30:00Z`, or with an offset
//!
//! An age is resolved when it is used, not when it is parsed, so a long-running
//! command does not drift.

use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, Utc};

/// A point in time, either fixed or relative to whenever it is asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Moment {
    /// An instant, as given.
    At(DateTime<Utc>),
    /// This long before now.
    Ago(Duration),
}

impl Moment {
    /// The instant this names, as of now.
    pub fn resolve(self) -> DateTime<Utc> {
        match self {
            Self::At(instant) => instant,
            Self::Ago(duration) => Utc::now() - duration,
        }
    }
}

impl fmt::Display for Moment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.resolve().to_rfc3339())
    }
}

impl FromStr for Moment {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.trim();
        if value.is_empty() {
            return Err("expected a date, a timestamp, or an age such as 7d".to_owned());
        }
        if value.eq_ignore_ascii_case("now") {
            return Ok(Self::Ago(Duration::zero()));
        }
        if let Some(duration) = parse_age(value) {
            return Ok(Self::Ago(duration));
        }
        // A full instant, with a zone.
        if let Ok(instant) = DateTime::parse_from_rfc3339(value) {
            return Ok(Self::At(instant.with_timezone(&Utc)));
        }
        // A date and time without a zone: taken as UTC, which is what CARP
        // stores in and so the least surprising reading of an unqualified one.
        for layout in [
            "%Y-%m-%dT%H:%M:%S",
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%dT%H:%M",
            "%Y-%m-%d %H:%M",
        ] {
            if let Ok(naive) = NaiveDateTime::parse_from_str(value, layout) {
                return Ok(Self::At(naive.and_utc()));
            }
        }
        // A bare date: from its first moment, so `--from 2026-08-01` includes
        // everything recorded that day.
        if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
            return Ok(Self::At(
                date.and_hms_opt(0, 0, 0).unwrap_or_default().and_utc(),
            ));
        }

        Err(format!(
            "{value} is not a date (2026-08-01), a timestamp (2026-08-01T09:30:00Z), \
             or an age (7d, 36h, 90m)"
        ))
    }
}

/// `7d`, `36h`, `90m`, `45s`, `2w` — a whole number and a unit.
fn parse_age(value: &str) -> Option<Duration> {
    let (digits, unit) = value.split_at(value.find(|c: char| !c.is_ascii_digit())?);
    if digits.is_empty() {
        return None;
    }
    let count: i64 = digits.parse().ok()?;
    // Guard the multiplication: `Duration::days` panics past its range, and a
    // typo like `99999999999d` should be an error message, not a crash.
    match unit.trim().to_ascii_lowercase().as_str() {
        "s" | "sec" | "secs" => Duration::try_seconds(count),
        "m" | "min" | "mins" => Duration::try_minutes(count),
        "h" | "hr" | "hrs" | "hour" | "hours" => Duration::try_hours(count),
        "d" | "day" | "days" => Duration::try_days(count),
        "w" | "week" | "weeks" => Duration::try_weeks(count),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(value: &str) -> DateTime<Utc> {
        match value.parse::<Moment>().unwrap() {
            Moment::At(instant) => instant,
            Moment::Ago(_) => panic!("{value} parsed as an age, not an instant"),
        }
    }

    #[test]
    fn an_instant_is_taken_as_given() {
        assert_eq!(
            at("2026-08-01T09:30:00Z").to_rfc3339(),
            "2026-08-01T09:30:00+00:00"
        );
        // An offset is honoured rather than ignored.
        assert_eq!(
            at("2026-08-01T11:30:00+02:00").to_rfc3339(),
            "2026-08-01T09:30:00+00:00"
        );
    }

    /// `--from 2026-08-01` has to include everything recorded on the first,
    /// so a bare date means its first moment.
    #[test]
    fn a_bare_date_starts_at_midnight_utc() {
        assert_eq!(at("2026-08-01").to_rfc3339(), "2026-08-01T00:00:00+00:00");
    }

    #[test]
    fn a_time_without_a_zone_is_read_as_utc() {
        for value in [
            "2026-08-01T09:30:00",
            "2026-08-01 09:30:00",
            "2026-08-01 09:30",
        ] {
            assert_eq!(
                at(value).to_rfc3339(),
                "2026-08-01T09:30:00+00:00",
                "{value}"
            );
        }
    }

    #[test]
    fn an_age_counts_back_from_now() {
        let seven_days = "7d".parse::<Moment>().unwrap();
        assert_eq!(seven_days, Moment::Ago(Duration::days(7)));

        let elapsed = Utc::now() - seven_days.resolve();
        assert!(
            (elapsed - Duration::days(7)).num_seconds().abs() < 5,
            "7d resolved to {elapsed} ago"
        );
    }

    #[test]
    fn every_age_unit_is_understood() {
        for (value, expected) in [
            ("45s", Duration::seconds(45)),
            ("90m", Duration::minutes(90)),
            ("36h", Duration::hours(36)),
            ("7d", Duration::days(7)),
            ("2w", Duration::weeks(2)),
            ("2weeks", Duration::weeks(2)),
            ("now", Duration::zero()),
        ] {
            assert_eq!(
                value.parse::<Moment>().unwrap(),
                Moment::Ago(expected),
                "{value}"
            );
        }
    }

    /// An age is resolved when used, not when parsed, so the window a
    /// long-running command asks for does not shift under it.
    #[test]
    fn an_age_stays_relative_until_it_is_used() {
        let moment = "1h".parse::<Moment>().unwrap();
        let first = moment.resolve();
        std::thread::sleep(std::time::Duration::from_millis(20));
        assert!(moment.resolve() > first, "an age resolved to a fixed point");
    }

    #[test]
    fn something_that_is_not_a_time_says_what_would_be() {
        let error = "last tuesday".parse::<Moment>().unwrap_err();
        assert!(error.contains("2026-08-01"), "{error}");
        assert!(error.contains("7d"), "{error}");

        assert!("".parse::<Moment>().is_err());
        assert!("7".parse::<Moment>().is_err());
        assert!("d".parse::<Moment>().is_err());
        assert!("7 fortnights".parse::<Moment>().is_err());
    }

    /// `Duration::days` panics beyond its range; a typo has to be an error.
    #[test]
    fn an_absurd_age_is_refused_rather_than_panicking() {
        assert!("99999999999999d".parse::<Moment>().is_err());
        assert!("9999999999999999999999w".parse::<Moment>().is_err());
    }
}
