// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Background work. Every function here spawns a task and reports back with a
//! [`Message`]; nothing blocks the render loop.

use std::path::PathBuf;

use tokio::sync::mpsc::UnboundedSender;

use crate::api::CarpClient;
use crate::api::endpoints::{accounts, exports, files, participants, studies};
use crate::api::models::{ParticipantQuery, SummaryExportRequest};
use crate::app::message::{LoadTarget, Message};
use crate::db::Cache;
use crate::download::{self, JobId};

type Tx = UnboundedSender<Message>;

/// Studies from the last session, so the list is populated instantly.
pub fn load_cached_studies(cache: Cache, tx: Tx) {
    tokio::spawn(async move {
        match cache.load_studies().await {
            Ok(studies) if !studies.is_empty() => {
                let _ = tx.send(Message::CachedStudies(studies));
            }
            Ok(_) => {}
            Err(error) => {
                let _ = tx.send(Message::Error(format!("cache unavailable: {error}")));
            }
        }
    });
}

pub fn load_studies(client: CarpClient, cache: Cache, tx: Tx) {
    tokio::spawn(async move {
        match studies::list(&client).await {
            Ok(studies) => {
                if let Err(error) = cache.save_studies(&studies).await {
                    let _ = tx.send(Message::Error(format!("could not cache studies: {error}")));
                }
                let _ = tx.send(Message::Studies(studies));
            }
            Err(error) => {
                let _ = tx.send(Message::LoadFailed {
                    study_id: None,
                    target: LoadTarget::Studies,
                    error: format!("loading studies: {error}"),
                });
            }
        }
    });
}

/// Researchers, assistants and participant group status of one study.
pub fn load_study_details(client: CarpClient, tx: Tx, study_id: String) {
    tokio::spawn(async move {
        let (researchers, assistants, status) = tokio::join!(
            studies::researchers(&client, &study_id),
            studies::research_assistants(&client, &study_id),
            studies::participant_group_status(&client, &study_id),
        );

        match (researchers, assistants) {
            (Ok(researchers), Ok(assistants)) => {
                let _ = tx.send(Message::Staff {
                    study_id: study_id.clone(),
                    researchers,
                    assistants,
                });
            }
            (Err(error), _) | (_, Err(error)) => {
                let _ = tx.send(Message::LoadFailed {
                    study_id: Some(study_id.clone()),
                    target: LoadTarget::Details,
                    error: format!("loading study staff: {error}"),
                });
            }
        }

        // Groups are informational; a failure there must not hide the staff.
        match status {
            Ok(status) => {
                let _ = tx.send(Message::Groups { study_id, status });
            }
            Err(error) => {
                let _ = tx.send(Message::Error(format!(
                    "loading participant groups: {error}"
                )));
            }
        }
    });
}

pub fn load_cached_participants(cache: Cache, tx: Tx, study_id: String) {
    tokio::spawn(async move {
        if let Ok(participants) = cache.load_participants(&study_id).await
            && !participants.is_empty()
        {
            let _ = tx.send(Message::CachedParticipants {
                study_id,
                participants,
            });
        }
    });
}

pub fn load_participants(
    client: CarpClient,
    cache: Cache,
    tx: Tx,
    study_id: String,
    query: ParticipantQuery,
) {
    tokio::spawn(async move {
        match participants::query(&client, &study_id, &query).await {
            Ok(page) => {
                // Only the unfiltered first page is worth caching.
                if query.page == 0 && query.search.is_none() {
                    let _ = cache.save_participants(&study_id, &page.content).await;
                }
                let _ = tx.send(Message::Participants { study_id, page });
            }
            Err(error) => {
                let _ = tx.send(Message::LoadFailed {
                    study_id: Some(study_id),
                    target: LoadTarget::Participants,
                    error: format!("loading participants: {error}"),
                });
            }
        }
    });
}

pub fn load_files(client: CarpClient, tx: Tx, study_id: String) {
    tokio::spawn(async move {
        match files::list(&client, &study_id, None).await {
            Ok(files) => {
                let _ = tx.send(Message::Files { study_id, files });
            }
            Err(error) => {
                let _ = tx.send(Message::LoadFailed {
                    study_id: Some(study_id),
                    target: LoadTarget::Files,
                    error: format!("loading files: {error}"),
                });
            }
        }
    });
}

pub fn load_exports(client: CarpClient, tx: Tx, study_id: String) {
    tokio::spawn(async move {
        match exports::list(&client, &study_id).await {
            Ok(exports) => {
                let _ = tx.send(Message::Exports { study_id, exports });
            }
            Err(error) => {
                let _ = tx.send(Message::LoadFailed {
                    study_id: Some(study_id),
                    target: LoadTarget::Exports,
                    error: format!("loading exports: {error}"),
                });
            }
        }
    });
}

/// Ask the server for a new study data export, then refresh the list.
pub fn request_export(client: CarpClient, tx: Tx, study_id: String) {
    tokio::spawn(async move {
        let request = SummaryExportRequest::default();
        match exports::request_summary(&client, &study_id, &request).await {
            Ok(()) => {
                let _ = tx.send(Message::Notice(
                    "export requested - it appears as 'in progress' until the server finishes"
                        .to_owned(),
                ));
                match exports::list(&client, &study_id).await {
                    Ok(exports) => {
                        let _ = tx.send(Message::Exports { study_id, exports });
                    }
                    Err(error) => {
                        let _ = tx.send(Message::Error(format!("loading exports: {error}")));
                    }
                }
            }
            Err(error) => {
                let _ = tx.send(Message::Error(format!("requesting export: {error}")));
            }
        }
    });
}

pub fn delete_export(client: CarpClient, tx: Tx, study_id: String, export_id: String) {
    tokio::spawn(async move {
        match exports::delete(&client, &study_id, &export_id).await {
            Ok(()) => {
                let _ = tx.send(Message::Notice("export deleted".to_owned()));
                if let Ok(exports) = exports::list(&client, &study_id).await {
                    let _ = tx.send(Message::Exports { study_id, exports });
                }
            }
            Err(error) => {
                let _ = tx.send(Message::Error(format!("deleting export: {error}")));
            }
        }
    });
}

/// Stream a file or export to the download directory.
pub fn download(
    client: CarpClient,
    tx: Tx,
    api_path: String,
    directory: PathBuf,
    file_name: String,
    job_id: JobId,
) {
    tokio::spawn(download::run(
        client, api_path, directory, file_name, job_id, tx,
    ));
}

/// Log a finished download so it survives a restart.
pub fn record_download(
    cache: Cache,
    study_id: Option<String>,
    label: String,
    path: PathBuf,
    bytes: u64,
) {
    tokio::spawn(async move {
        let _ = cache
            .record_download(study_id.as_deref(), &label, &path, bytes)
            .await;
    });
}

/// Ask the server where its web clients live, so `o` can open a study in the
/// browser. Best effort: a failure just leaves the assumed address in place.
pub fn discover_portal(client: CarpClient, tx: Tx) {
    tokio::spawn(async move {
        if let Ok(uris) = accounts::redirect_uris(&client).await
            && let Some(origin) = crate::portal::origin_from_redirect_uris(&uris)
        {
            let _ = tx.send(Message::PortalDiscovered(origin));
        }
    });
}

/// Previously downloaded files, for the downloads screen.
pub fn load_download_history(cache: Cache, tx: Tx) {
    tokio::spawn(async move {
        if let Ok(records) = cache.recent_downloads(20).await
            && !records.is_empty()
        {
            let _ = tx.send(Message::DownloadHistory(records));
        }
    });
}
