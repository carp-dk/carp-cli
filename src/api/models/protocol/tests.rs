// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`].

use super::*;

fn protocol() -> carp_protocol::StudyProtocol {
    carp_protocol::StudyProtocol::new("Sleep", "979b408d-784e-4b1b-bb1e-ff9204e072f3")
}

/// The discriminator is how CAWS dispatches the command, so the exact
/// class name matters more than anything else in the payload.
#[test]
fn a_first_upload_is_an_add_command() {
    let request = ProtocolRequest::store(protocol(), "v1.0.0".to_owned(), true);
    let json = serde_json::to_value(&request).unwrap();

    assert_eq!(
        json["__type"],
        "dk.cachet.carp.protocols.infrastructure.ProtocolServiceRequest.Add"
    );
    assert_eq!(json["versionTag"], "v1.0.0");
    assert_eq!(json["apiVersion"]["major"], 1);
}

/// A later upload has to be `AddVersion`, or CAWS rejects it as a
/// duplicate rather than filing it as a new revision.
#[test]
fn a_later_upload_is_an_add_version_command() {
    let request = ProtocolRequest::store(protocol(), "v1.1.0".to_owned(), false);
    let json = serde_json::to_value(&request).unwrap();

    assert_eq!(
        json["__type"],
        "dk.cachet.carp.protocols.infrastructure.ProtocolServiceRequest.AddVersion"
    );
}

/// The protocol travels as itself, not as a nested string, so the
/// serialisation this crate is built around has to survive the wrapper.
#[test]
fn the_protocol_is_embedded_whole() {
    let request = ProtocolRequest::store(protocol(), "v1.0.0".to_owned(), true);
    let json = serde_json::to_value(&request).unwrap();

    assert_eq!(json["protocol"]["name"], "Sleep");
    assert!(json["protocol"]["primaryDevices"].is_array());
}

#[test]
fn an_overview_reads_its_name_out_of_the_snapshot() {
    let overview: ProtocolOverview = serde_json::from_value(serde_json::json!({
        "versionTag": "v1.0.0",
        "snapshot": { "name": "Sleep and mood", "id": "bf9eb630-73f7-11ee-9063-9df29e0fafa1" }
    }))
    .unwrap();

    assert_eq!(overview.name(), "Sleep and mood");
    assert_eq!(overview.id(), Some("bf9eb630-73f7-11ee-9063-9df29e0fafa1"));
}

/// A listing entry with no snapshot must not panic the list that shows it.
#[test]
fn an_overview_without_a_snapshot_still_reads() {
    let overview = ProtocolOverview::default();
    assert_eq!(overview.name(), "unnamed protocol");
    assert_eq!(overview.id(), None);
}
