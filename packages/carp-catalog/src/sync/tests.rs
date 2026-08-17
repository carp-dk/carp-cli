// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`].

use super::*;
use crate::source::ProtocolDocument;

fn commit(sha: &str) -> Commit {
    Commit {
        sha: sha.to_owned(),
        date: "2026-07-07T13:52:26+02:00".to_owned(),
        subject: "Update NT to protocol v2".to_owned(),
    }
}

fn snapshot(sha: &str) -> Snapshot {
    Snapshot::new(
        "carp-dk/carp_study_app_configurations".to_owned(),
        commit(sha),
        vec![ProtocolDocument {
            study: "neuropathy".to_owned(),
            path: "neuropathy/carp/resources/protocol.json".to_owned(),
            json: include_str!("../../../carp-protocol/tests/corpus/neuropathy.json").to_owned(),
        }],
    )
}

/// The editor loads the catalogue at startup and must not need a network.
#[tokio::test]
async fn a_stored_catalogue_loads_without_the_network() {
    let directory = std::env::temp_dir().join("carp-catalog-load-test");
    let _ = tokio::fs::remove_dir_all(&directory).await;

    snapshot("158cdcbe94980d48afc6478dd8f11c7ac4bbff5a")
        .save(&directory)
        .await
        .unwrap();

    let catalog = load(&directory).await.unwrap();
    assert_eq!(catalog.version.unwrap().studies, 1);
    assert!(!catalog.measure_types.is_empty());

    tokio::fs::remove_dir_all(&directory).await.unwrap();
}

#[tokio::test]
async fn loading_without_a_catalogue_reports_missing() {
    let directory = std::env::temp_dir().join("carp-catalog-missing-test");
    let _ = tokio::fs::remove_dir_all(&directory).await;

    assert!(matches!(load(&directory).await, Err(Error::Missing)));
}

#[test]
fn a_report_reads_as_a_sentence() {
    let mut catalog = Catalog::default();
    catalog.version = Some(crate::catalog::CatalogVersion {
        repository: "carp-dk/carp_study_app_configurations".to_owned(),
        commit: commit("158cdcbe94980d48afc6478dd8f11c7ac4bbff5a"),
        fetched_at: chrono::Utc::now().to_rfc3339(),
        studies: 10,
    });

    let updated = SyncReport {
        outcome: SyncOutcome::Updated,
        commit: commit("158cdcbe94980d48afc6478dd8f11c7ac4bbff5a"),
        previous: Some(commit("ab6348b0000000000000000000000000000000000")),
        catalog: catalog.clone(),
    };
    assert_eq!(
        updated.summary(),
        "catalogue updated: ab6348b → 158cdcb (10 studies)"
    );

    let created = SyncReport {
        outcome: SyncOutcome::Created,
        commit: commit("158cdcbe94980d48afc6478dd8f11c7ac4bbff5a"),
        previous: None,
        catalog,
    };
    assert_eq!(
        created.summary(),
        "catalogue downloaded at 158cdcb (10 studies)"
    );
}
