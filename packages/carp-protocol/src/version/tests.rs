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
fn version_tags_parse_with_or_without_the_v() {
    assert_eq!(VersionTag("v1.2.3".to_owned()).parts(), Some((1, 2, 3)));
    assert_eq!(VersionTag("1.2.3".to_owned()).parts(), Some((1, 2, 3)));
    assert_eq!(VersionTag::initial().parts(), Some((1, 0, 0)));
}

/// A tag that is not semantic must not be silently rewritten into one.
#[test]
fn an_unrecognised_tag_cannot_be_incremented() {
    for tag in ["pilot-2", "", "v1", "v1.2", "v1.2.3.4", "va.b.c"] {
        let tag = VersionTag(tag.to_owned());
        assert_eq!(tag.parts(), None, "{tag} should not parse");
        assert_eq!(tag.next(Bump::Patch), None);
    }
}

#[test]
fn bumping_resets_the_components_below_it() {
    let tag = VersionTag("v1.2.3".to_owned());
    assert_eq!(tag.next(Bump::Patch).unwrap().as_str(), "v1.2.4");
    assert_eq!(tag.next(Bump::Minor).unwrap().as_str(), "v1.3.0");
    assert_eq!(tag.next(Bump::Major).unwrap().as_str(), "v2.0.0");
}

/// A new revision keeps the id: that is what makes CAWS file it as a
/// version of the same protocol rather than a new one.
#[test]
fn a_new_revision_keeps_the_identity() {
    let mut protocol = StudyProtocol::new("Sleep", "979b408d-784e-4b1b-bb1e-ff9204e072f3");
    let id = protocol.id.clone();

    assert_eq!(next_revision(&mut protocol), 1);
    assert_eq!(next_revision(&mut protocol), 2);
    assert_eq!(protocol.id, id);
}

/// A fork must not keep the id, or uploading it would rewrite the
/// original's history rather than starting its own.
#[test]
fn a_fork_takes_a_new_identity() {
    let mut protocol = StudyProtocol::new("Sleep", "979b408d-784e-4b1b-bb1e-ff9204e072f3");
    next_revision(&mut protocol);
    let id = protocol.id.clone();

    fork(&mut protocol, "Sleep (pilot)");

    assert_ne!(protocol.id, id);
    assert_eq!(protocol.version, 0);
    assert_eq!(protocol.name, "Sleep (pilot)");
}

/// The placeholder owner id the ICAT protocol carries is fine in a source
/// document and fatal on upload; the two checks must disagree about it.
#[test]
fn a_placeholder_owner_blocks_upload_but_not_validation() {
    let mut protocol = StudyProtocol::new("ICAT", "to_be_set_on_upload");
    crate::builder::add_device(&mut protocol, crate::device::DeviceKind::Smartphone);

    let errors = validate(&protocol)
        .into_iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
        .count();
    assert_eq!(errors, 0, "a placeholder owner is legal in a document");

    let check = UploadCheck::run(&protocol);
    assert!(!check.is_ready());
    assert!(
        check.blockers.iter().any(|blocker| blocker.contains("owner id")),
        "{:?}",
        check.blockers
    );
}

#[test]
fn a_sound_protocol_is_ready_to_upload() {
    let mut protocol = StudyProtocol::new("Sleep", "979b408d-784e-4b1b-bb1e-ff9204e072f3");
    crate::builder::add_device(&mut protocol, crate::device::DeviceKind::Smartphone);

    assert!(UploadCheck::run(&protocol).is_ready());
}
