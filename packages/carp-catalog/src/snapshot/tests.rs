// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`].

use super::*;

fn snapshot() -> Snapshot {
    Snapshot::new(
        "carp-dk/carp_study_app_configurations".to_owned(),
        Commit {
            sha: "158cdcbe94980d48afc6478dd8f11c7ac4bbff5a".to_owned(),
            date: "2026-07-07T13:52:26+02:00".to_owned(),
            subject: "Update NT to protocol v2".to_owned(),
        },
        vec![ProtocolDocument {
            study: "neuropathy".to_owned(),
            path: "neuropathy/carp/resources/protocol.json".to_owned(),
            json: include_str!("../../../carp-protocol/tests/corpus/neuropathy.json").to_owned(),
        }],
    )
}

#[tokio::test]
async fn a_snapshot_survives_a_save_and_load() {
    let directory = std::env::temp_dir().join("carp-catalog-snapshot-test");
    let _ = tokio::fs::remove_dir_all(&directory).await;

    let original = snapshot();
    original.save(&directory).await.unwrap();
    let loaded = Snapshot::load(&directory).await.unwrap();

    assert_eq!(loaded, original);
    assert_eq!(loaded.study_names(), ["neuropathy"]);

    tokio::fs::remove_dir_all(&directory).await.unwrap();
}

#[tokio::test]
async fn an_absent_snapshot_reports_missing_rather_than_failing() {
    let directory = std::env::temp_dir().join("carp-catalog-absent-test");
    let _ = tokio::fs::remove_dir_all(&directory).await;

    assert!(matches!(
        Snapshot::load(&directory).await,
        Err(Error::Missing)
    ));
}

/// A snapshot from a build with a different layout must be re-synced, not
/// read as if it were current.
#[tokio::test]
async fn a_snapshot_from_another_layout_is_discarded() {
    let directory = std::env::temp_dir().join("carp-catalog-version-test");
    let _ = tokio::fs::remove_dir_all(&directory).await;

    let mut stale = snapshot();
    stale.format_version = FORMAT_VERSION + 1;
    stale.save(&directory).await.unwrap();

    assert!(matches!(
        Snapshot::load(&directory).await,
        Err(Error::Missing)
    ));

    tokio::fs::remove_dir_all(&directory).await.unwrap();
}

/// The templates the editor offers must actually load as protocols.
#[test]
fn a_study_can_be_taken_as_a_template() {
    let snapshot = snapshot();
    let protocol = snapshot.template("neuropathy").unwrap();

    assert_eq!(protocol.name, "CARP Neuropathy Tracker Protocol");
    assert!(snapshot.template("nonexistent").is_err());
}

/// One unparseable document must not sink the whole catalogue.
#[test]
fn a_malformed_document_is_skipped_not_fatal() {
    let mut snapshot = snapshot();
    snapshot.documents.push(ProtocolDocument {
        study: "broken".to_owned(),
        path: "broken/carp/resources/protocol.json".to_owned(),
        json: "{ not json".to_owned(),
    });

    let (parsed, failed) = snapshot.parsed();
    assert_eq!(parsed.len(), 1);
    assert_eq!(failed.len(), 1);
    assert!(failed[0].starts_with("broken:"));
}
