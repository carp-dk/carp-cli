// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`].

use super::*;
use crate::device::Device;

/// Every kind must serialise under the discriminator it advertises, or a
/// device created from the picker would not be the one the label promised.
#[test]
fn every_kind_serialises_as_its_own_type_name() {
    for kind in DeviceKind::ALL {
        let device = kind.instantiate(kind.default_role_name().to_owned());
        let json = serde_json::to_value(&device).unwrap();
        assert_eq!(
            json["__type"].as_str(),
            Some(kind.type_name()),
            "{} serialised as {}",
            kind.label(),
            json["__type"]
        );
        assert_eq!(DeviceKind::from_type_name(kind.type_name()), Some(kind));
    }
}

/// A created device must read back as the kind that made it, with the
/// primary/connected split and optionality the class implies.
#[test]
fn a_created_device_reads_back_unchanged() {
    for kind in DeviceKind::ALL {
        let device = kind.instantiate("Role".to_owned());
        let json = serde_json::to_string(&device).unwrap();
        let parsed: Device = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed, device, "{} did not round trip", kind.label());
        assert_eq!(parsed.kind(), Some(kind));
        assert_eq!(parsed.is_primary(), kind.is_primary());
        assert_eq!(
            parsed.is_optional(),
            !kind.is_primary(),
            "{} optionality",
            kind.label()
        );
    }
}

/// A type name has to belong to exactly one kind, or `from_type_name`
/// would resolve a document to the wrong class.
#[test]
fn type_names_are_unique() {
    let mut names: Vec<&str> = DeviceKind::ALL
        .iter()
        .map(|kind| kind.type_name())
        .collect();
    names.sort_unstable();
    let count = names.len();
    names.dedup();
    assert_eq!(names.len(), count, "two kinds share a discriminator");
}

/// The namespaces are what a protocol's API level selects between.
#[test]
fn kinds_know_their_namespace() {
    assert_eq!(DeviceKind::Smartphone.namespace(), Namespace::Core);
    assert_eq!(DeviceKind::Cams2Smartphone.namespace(), Namespace::Cams2);
    assert_eq!(Namespace::for_api_level(Some("2.0")), Namespace::Cams2);
    assert_eq!(Namespace::for_api_level(None), Namespace::Core);
    assert_eq!(Namespace::Cams2.api_level(), Some("2.0"));
}

/// A class present in both namespaces has to map across; one present in
/// only one has to say so rather than resolving to something else.
#[test]
fn classes_map_between_namespaces() {
    assert_eq!(
        DeviceKind::Smartphone.in_namespace(Namespace::Cams2),
        Some(DeviceKind::Cams2Smartphone)
    );
    assert_eq!(
        DeviceKind::Cams2PolarDevice.in_namespace(Namespace::Core),
        Some(DeviceKind::PolarDevice)
    );
    // These two exist only under the original namespace.
    assert_eq!(DeviceKind::WebBrowser.in_namespace(Namespace::Cams2), None);
    assert_eq!(
        DeviceKind::CortriumDevice.in_namespace(Namespace::Cams2),
        None
    );
}

#[test]
fn both_smartphone_kinds_are_primary() {
    assert!(DeviceKind::Smartphone.is_primary());
    assert!(DeviceKind::Cams2Smartphone.is_primary());
    assert_ne!(
        DeviceKind::Smartphone.type_name(),
        DeviceKind::Cams2Smartphone.type_name()
    );
}
