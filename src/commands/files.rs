// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! `carp files` - what participants and researchers uploaded to a study.

use carp_client::ApiError;
use carp_client::api::endpoints::files;
use carp_client::api::models::{StudyFile, format_instant};
use color_eyre::Result;

use crate::cli::{FilesCommand, Global};
use crate::commands::{Session, connect};
use crate::output::{self, Rows};
use crate::transfer;

pub async fn run(command: &FilesCommand, global: &Global) -> Result<()> {
    let session = connect(global).await?;
    match command {
        FilesCommand::List { study, query } => list(&session, study, query.as_deref()).await,
        FilesCommand::Download { study, file } => download(&session, study, *file).await,
    }
}

impl Rows for StudyFile {
    // The listing carries no size: `GET /files` describes what a file is and
    // where it came from, not how large it is.
    const HEADERS: &'static [&'static str] = &["id", "name", "deployment", "uploaded", "by"];

    fn cells(&self) -> Vec<String> {
        vec![
            self.id.to_string(),
            self.download_name().to_owned(),
            self.deployment_label().to_owned(),
            format_instant(self.created_at),
            self.created_by.clone().unwrap_or_else(|| "-".to_owned()),
        ]
    }
}

async fn list(session: &Session, study: &str, query: Option<&str>) -> Result<()> {
    let files = files::list(&session.client, study, query).await?;
    output::rows(&files, session.format)
}

async fn download(session: &Session, study: &str, file_id: i32) -> Result<()> {
    let file = files::list(&session.client, study, None)
        .await?
        .into_iter()
        .find(|file| file.id == file_id)
        .ok_or_else(|| ApiError::NotFound(format!("no file {file_id} in study {study}")))?;

    transfer::to_disk(
        session,
        &files::download_path(study, file_id),
        file.download_name(),
        Some(study.to_owned()),
    )
    .await
}
