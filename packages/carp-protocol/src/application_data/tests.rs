// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`].

use super::*;

/// The endpoint the reference protocols use must survive a round trip
/// with every field intact, defaults included.
#[test]
fn the_carp_endpoint_round_trips() {
    let original = serde_json::json!({
        "__type": "CarpDataEndPoint",
        "type": "CAWS",
        "dataFormat": "dk.cachet.carp",
        "uploadMethod": "stream",
        "name": "CARP Web Service",
        "onlyUploadOnWiFi": false,
        "uploadInterval": 10,
        "deleteWhenUploaded": false,
        "compress": true
    });

    let endpoint: DataEndPoint = serde_json::from_value(original.clone()).unwrap();
    assert_eq!(serde_json::to_value(&endpoint).unwrap(), original);
    assert_eq!(endpoint.label(), "CARP Web Service (stream)");
}

/// `onlyUploadOnWiFi` is not camel case in the way serde would guess, so
/// it is worth pinning: a silent rename would upload on cellular data.
#[test]
fn the_wifi_flag_keeps_its_exact_spelling() {
    let json = serde_json::to_value(DataEndPoint::carp_stream()).unwrap();
    assert!(
        json.get("onlyUploadOnWiFi").is_some(),
        "spelled onlyUploadOnWiFi on the wire, got {json}"
    );
}

/// An endpoint type added upstream must not stop a protocol loading.
#[test]
fn an_unmodelled_endpoint_survives() {
    let original = serde_json::json!({
        "__type": "S3DataEndPoint",
        "bucket": "carp-uploads"
    });
    let endpoint: DataEndPoint = serde_json::from_value(original.clone()).unwrap();
    assert!(matches!(endpoint, DataEndPoint::Unknown(_)));
    assert_eq!(serde_json::to_value(&endpoint).unwrap(), original);
}
