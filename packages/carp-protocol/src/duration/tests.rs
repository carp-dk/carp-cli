// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`].

use super::*;

/// The constants that appear literally in the reference protocols.
#[test]
fn the_wire_values_mean_what_the_protocols_intend() {
    assert_eq!(Micros::from_days(1), Micros(86_400_000_000));
    assert_eq!(Micros::from_seconds(60), Micros(60_000_000));
    assert_eq!(Micros::from_days(30), Micros(2_592_000_000_000));
    assert_eq!(Micros::from_days(5), Micros(432_000_000_000));
}

/// A duration serialises as the bare integer, with no wrapper object.
#[test]
fn serialisation_is_a_plain_integer() {
    let json = serde_json::to_string(&Micros::from_days(1)).unwrap();
    assert_eq!(json, "86400000000");
    assert_eq!(
        serde_json::from_str::<Micros>("86400000000").unwrap(),
        Micros::from_days(1)
    );
}

#[test]
fn human_durations_parse() {
    assert_eq!(Micros::parse("30d"), Some(Micros::from_days(30)));
    assert_eq!(Micros::parse("1h30m"), Some(Micros(90 * PER_MINUTE)));
    assert_eq!(Micros::parse("1h 30m"), Some(Micros(90 * PER_MINUTE)));
    assert_eq!(Micros::parse("500ms"), Some(Micros(500_000)));
    assert_eq!(Micros::parse("250us"), Some(Micros(250)));
    // A bare number is the seconds a "period" field is usually meant in.
    assert_eq!(Micros::parse("90"), Some(Micros::from_seconds(90)));
}

#[test]
fn nonsense_does_not_parse() {
    for input in ["", "  ", "d", "1x", "1h2", "abc", "1..2"] {
        assert_eq!(Micros::parse(input), None, "{input:?} should not parse");
    }
    // A number and its unit belong together.
    for input in ["1 h", "30 d"] {
        assert_eq!(Micros::parse(input), None, "{input:?} should not parse");
    }
}

/// Rendering must be lossless: whatever `human` prints has to read back as
/// the same duration, or the editor would corrupt a value just by showing
/// and re-saving it.
#[test]
fn rendering_round_trips_through_parsing() {
    for micros in [
        0,
        1,
        999,
        1_000,
        PER_SECOND,
        90 * PER_MINUTE,
        PER_DAY,
        2_592_000_000_000,
        432_000_000_000,
        PER_DAY + PER_HOUR + PER_MINUTE + PER_SECOND + 1_500,
    ] {
        let duration = Micros(micros);
        let rendered = duration.human();
        assert_eq!(
            Micros::parse(&rendered),
            Some(duration),
            "{rendered:?} did not read back as {micros}"
        );
    }
}

#[test]
fn familiar_values_read_well() {
    assert_eq!(Micros::from_days(30).human(), "30d");
    assert_eq!(Micros::from_seconds(60).human(), "1m");
    assert_eq!(Micros(0).human(), "0s");
    assert_eq!(Micros(1_500).human(), "1ms 500us");
}
