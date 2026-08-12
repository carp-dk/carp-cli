// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`].

use super::*;

#[test]
fn times_parse_and_render() {
    assert_eq!(TimeOfDay::parse("20:00"), Some(TimeOfDay::new(20, 0)));
    assert_eq!(TimeOfDay::new(20, 0).label(), "20:00");
    assert_eq!(
        TimeOfDay::parse("08:05:30"),
        Some(TimeOfDay {
            hour: 8,
            minute: 5,
            second: 30
        })
    );
    assert_eq!(
        TimeOfDay {
            hour: 8,
            minute: 5,
            second: 30
        }
        .label(),
        "08:05:30"
    );
}

#[test]
fn impossible_times_are_rejected() {
    for input in ["24:00", "10:60", "10:00:60", "10", "", "a:b", "1:2:3:4"] {
        assert_eq!(TimeOfDay::parse(input), None, "{input:?} should not parse");
    }
}

/// The two representations of a recurrence must agree, or CAMS schedules
/// on the period while the editor shows the type.
#[test]
fn each_recurrence_matches_its_period() {
    assert_eq!(Recurrence::Daily.period(), Micros::from_days(1));
    assert_eq!(Recurrence::Weekly.period(), Micros::from_days(7));
    // The value the reference protocols write for a weekly trigger.
    assert_eq!(Recurrence::Weekly.period().micros(), 604_800_000_000);
    for recurrence in Recurrence::ALL {
        assert_eq!(Recurrence::parse(recurrence.wire_name()), Some(recurrence));
    }
}

#[test]
fn days_of_week_are_iso_numbered() {
    assert_eq!(DayOfWeek::MONDAY.0, 1);
    assert_eq!(DayOfWeek::SUNDAY.label(), "Sunday");
    assert!(!DayOfWeek(0).is_valid());
    assert!(!DayOfWeek(8).is_valid());
}
