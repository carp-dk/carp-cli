// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! `carp export` - the bulk path to a study's data.
//!
//! Exports are asynchronous: the server packages one in the background, so
//! `create` returns before there is anything to download. `--wait` polls until
//! there is.

use std::time::Duration;

use carp_client::ApiError;
use carp_client::api::endpoints::exports;
use carp_client::api::models::{CarpUuid, Export, SummaryExportRequest, format_instant};
use color_eyre::Result;
use color_eyre::eyre::eyre;

use crate::cli::{ExportCommand, Global};
use crate::commands::{Session, connect};
use crate::output::{self, Rows};
use crate::transfer;

/// How often `--wait` asks whether the archive is ready, and for how long.
/// A study export is minutes of server work, so polling faster would only add
/// load; giving up after an hour keeps a wedged job from hanging a CI step.
const POLL_INTERVAL: Duration = Duration::from_secs(10);
const POLL_LIMIT: Duration = Duration::from_secs(60 * 60);

pub async fn run(command: &ExportCommand, global: &Global) -> Result<()> {
    let session = connect(global).await?;
    match command {
        ExportCommand::List { study } => list(&session, study).await,
        ExportCommand::Create {
            study,
            deployment,
            active_only,
            wait,
        } => create(&session, study, deployment, *active_only, *wait).await,
        ExportCommand::Download { study, export } => download(&session, study, export).await,
        ExportCommand::Delete { study, export } => delete(&session, study, export).await,
    }
}

impl Rows for Export {
    const HEADERS: &'static [&'static str] = &["id", "file", "status", "kind", "created", "by"];

    fn cells(&self) -> Vec<String> {
        vec![
            self.id.clone(),
            self.display_name(),
            self.status.label().to_owned(),
            self.kind.label().to_owned(),
            format_instant(self.created_at),
            self.created_by.clone().unwrap_or_else(|| "-".to_owned()),
        ]
    }
}

async fn list(session: &Session, study: &str) -> Result<()> {
    let exports = exports::list(&session.client, study).await?;
    output::rows(&exports, session.format)
}

async fn create(
    session: &Session,
    study: &str,
    deployments: &[String],
    active_only: bool,
    wait: bool,
) -> Result<()> {
    // The set of exports before the request, so the new one can be told from
    // the ones already there. The server does not return the id it created.
    let before: Vec<String> = exports::list(&session.client, study)
        .await?
        .into_iter()
        .map(|export| export.id)
        .collect();

    let request = SummaryExportRequest {
        deployment_ids: (!deployments.is_empty())
            .then(|| deployments.iter().cloned().map(CarpUuid::new).collect()),
        active_deployments_only: active_only.then_some(true),
    };
    exports::request_summary(&session.client, study, &request).await?;
    output::note("export requested; the server is packaging it");

    let Some(created) = appeared(session, study, &before).await? else {
        // Requested successfully but not yet listed. Not a failure: it will
        // appear. Say so rather than inventing a row.
        output::note("it is not listed yet - `carp export list` will show it shortly");
        return Ok(());
    };

    if !wait {
        return output::rows(std::slice::from_ref(&created), session.format);
    }

    let ready = poll_until_ready(session, study, &created.id).await?;
    output::rows(std::slice::from_ref(&ready), session.format)
}

/// The export in the study's list that was not there before.
async fn appeared(session: &Session, study: &str, before: &[String]) -> Result<Option<Export>> {
    Ok(exports::list(&session.client, study)
        .await?
        .into_iter()
        .find(|export| !before.contains(&export.id)))
}

async fn poll_until_ready(session: &Session, study: &str, export_id: &str) -> Result<Export> {
    let started = std::time::Instant::now();
    loop {
        let export = find(session, study, export_id).await?;
        if export.status.is_downloadable() {
            output::note(format!("{} is ready", export.display_name()));
            return Ok(export);
        }
        if !export.status.is_pending() {
            return Err(eyre!(
                "export {export_id} ended as {}",
                export.status.label()
            ));
        }
        if started.elapsed() > POLL_LIMIT {
            return Err(eyre!(
                "export {export_id} was still {} after {} minutes",
                export.status.label(),
                POLL_LIMIT.as_secs() / 60
            ));
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

async fn find(session: &Session, study: &str, export_id: &str) -> Result<Export> {
    exports::list(&session.client, study)
        .await?
        .into_iter()
        .find(|export| export.id == export_id)
        .ok_or_else(|| ApiError::NotFound(format!("no export {export_id} in study {study}")).into())
}

async fn download(session: &Session, study: &str, export_id: &str) -> Result<()> {
    let export = find(session, study, export_id).await?;
    if !export.status.is_downloadable() {
        return Err(eyre!(
            "export {export_id} is {} - only a finished export can be downloaded",
            export.status.label()
        ));
    }

    transfer::to_disk(
        session,
        &exports::download_path(study, export_id),
        &export.display_name(),
        Some(study.to_owned()),
    )
    .await
}

async fn delete(session: &Session, study: &str, export_id: &str) -> Result<()> {
    exports::delete(&session.client, study, export_id).await?;
    output::note(format!("deleted export {export_id}"));
    Ok(())
}
