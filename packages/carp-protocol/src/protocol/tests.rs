// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`].

use super::*;

#[test]
fn a_new_protocol_has_a_fresh_identity() {
    let first = StudyProtocol::new("Sleep", "979b408d-784e-4b1b-bb1e-ff9204e072f3");
    let second = StudyProtocol::new("Sleep", "979b408d-784e-4b1b-bb1e-ff9204e072f3");

    assert_ne!(first.id, second.id, "each protocol needs its own id");
    assert_eq!(first.version, 0);
    assert!(uuid::Uuid::parse_str(&first.id).is_ok());
}

/// Trigger ids are written as JSON strings but ordered as numbers, so
/// trigger 10 must come after trigger 2 rather than after trigger 1.
#[test]
fn trigger_ids_serialise_as_numerically_ordered_string_keys() {
    let mut protocol = StudyProtocol::new("Ordering", "owner");
    for id in [0, 1, 2, 10] {
        protocol
            .triggers
            .insert(id, Trigger::immediate("Primary Phone"));
    }

    let json = serde_json::to_value(&protocol).unwrap();
    let keys: Vec<&String> = json["triggers"].as_object().unwrap().keys().collect();
    assert_eq!(keys, ["0", "1", "2", "10"]);
}

/// A gap left by deleting a trigger is reused rather than skipped.
#[test]
fn the_next_trigger_id_fills_gaps() {
    let mut protocol = StudyProtocol::new("Gaps", "owner");
    assert_eq!(protocol.next_trigger_id(), 0);

    for id in [0, 1, 2] {
        protocol
            .triggers
            .insert(id, Trigger::immediate("Primary Phone"));
    }
    assert_eq!(protocol.next_trigger_id(), 3);

    protocol.triggers.remove(&1);
    assert_eq!(protocol.next_trigger_id(), 1);
}
