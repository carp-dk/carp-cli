// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`].

use super::*;

/// The health window is where a microsecond/millisecond mix-up would be
/// invisible and wrong, so the wire values are pinned.
#[test]
fn the_health_window_uses_the_wire_units() {
    let configuration = SamplingConfiguration::health(vec!["STEPS".to_owned()]);
    let json = serde_json::to_value(&configuration).unwrap();

    assert_eq!(json["past"], 2_592_000_000_000i64, "30 days in microseconds");
    assert_eq!(json["future"], 86_400_000_000i64, "1 day in microseconds");
    assert_eq!(json["healthDataTypes"], serde_json::json!(["STEPS"]));
}

#[test]
fn a_health_configuration_round_trips() {
    let original = serde_json::json!({
        "__type": "dk.cachet.carp.common.application.sampling.HealthSamplingConfiguration",
        "past": 2_592_000_000_000i64,
        "future": 86_400_000_000i64,
        "healthDataTypes": ["WEIGHT", "HEIGHT", "STEPS"]
    });

    let configuration: SamplingConfiguration = serde_json::from_value(original.clone()).unwrap();
    assert_eq!(configuration.health_data_types().unwrap().len(), 3);
    assert_eq!(serde_json::to_value(&configuration).unwrap(), original);
}

#[test]
fn a_location_configuration_round_trips() {
    let original = serde_json::json!({
        "__type": "dk.cachet.carp.common.application.sampling.LocationSamplingConfiguration",
        "once": true
    });

    let configuration: SamplingConfiguration = serde_json::from_value(original.clone()).unwrap();
    assert_eq!(configuration.label(), "single fix");
    assert_eq!(serde_json::to_value(&configuration).unwrap(), original);
}
