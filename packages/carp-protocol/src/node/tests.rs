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
fn a_leaf_class_name_is_the_last_segment() {
    assert_eq!(
        short_type("dk.cachet.carp.common.application.devices.PolarDevice"),
        "PolarDevice"
    );
}

/// `DataStream` on its own says nothing; `Measure.DataStream` does.
#[test]
fn a_nested_class_keeps_its_parent() {
    assert_eq!(
        short_type("dk.cachet.carp.common.application.tasks.Measure.DataStream"),
        "Measure.DataStream"
    );
    assert_eq!(
        short_type("dk.cachet.carp.common.application.users.AssignedTo.Roles"),
        "AssignedTo.Roles"
    );
}

/// Types that are already short, and lower-case-only paths, must not panic.
#[test]
fn degenerate_names_are_handled() {
    assert_eq!(short_type("RPChoice"), "RPChoice");
    assert_eq!(short_type("dk.cachet.carp.movesense.state"), "state");
    assert_eq!(short_type(""), "");
}

/// The whole point: an unmodelled node survives a load/save cycle.
#[test]
fn an_unknown_node_round_trips_verbatim() {
    let original = serde_json::json!({
        "__type": "dk.cachet.carp.common.application.devices.FutureSensor",
        "roleName": "Future Sensor",
        "isOptional": true,
        "nested": { "samplingRate": 512 }
    });

    let node: UnknownNode = serde_json::from_value(original.clone()).unwrap();
    assert_eq!(node.short_type(), "FutureSensor");
    assert_eq!(node.role_name(), Some("Future Sensor"));
    assert_eq!(serde_json::to_value(&node).unwrap(), original);
}
