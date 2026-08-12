// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`].

use super::*;

/// The common case writes two fields and no null override, because CARP
/// treats a present-but-null override differently from an absent one.
#[test]
fn a_plain_measure_omits_the_override() {
    let measure = Measure::data_stream("dk.cachet.carp.stepcount");
    let json = serde_json::to_value(&measure).unwrap();

    assert_eq!(
        json,
        serde_json::json!({
            "__type": "dk.cachet.carp.common.application.tasks.Measure.DataStream",
            "type": "dk.cachet.carp.stepcount"
        })
    );
    assert_eq!(measure.short_name(), "stepcount");
}

#[test]
fn an_overridden_measure_round_trips() {
    let original = serde_json::json!({
        "__type": "dk.cachet.carp.common.application.tasks.Measure.DataStream",
        "type": "dk.cachet.carp.health",
        "overrideSamplingConfiguration": {
            "__type": "dk.cachet.carp.common.application.sampling.HealthSamplingConfiguration",
            "past": 2_592_000_000_000i64,
            "future": 86_400_000_000i64,
            "healthDataTypes": ["WEIGHT"]
        }
    });

    let measure: Measure = serde_json::from_value(original.clone()).unwrap();
    assert!(measure.sampling().is_some());
    assert_eq!(serde_json::to_value(&measure).unwrap(), original);
}
