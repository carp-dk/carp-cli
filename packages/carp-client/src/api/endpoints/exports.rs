// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Export endpoints (`export-controller`).
//!
//! Exports are asynchronous: request one, poll the list until its status is
//! `AVAILABLE`, then download the archive.

use crate::api::client::CarpClient;
use crate::api::error::ApiResult;
use crate::api::models::{Export, SummaryExportRequest};

/// Every export belonging to a study, newest first.
pub async fn list(client: &CarpClient, study_id: &str) -> ApiResult<Vec<Export>> {
    let mut exports: Vec<Export> = client
        .get_json(&format!("/api/studies/{study_id}/exports"), &[])
        .await?;
    exports.sort_by_key(|export| std::cmp::Reverse(export.created_at));
    Ok(exports)
}

/// Ask the server to build a study data export.
pub async fn request_summary(
    client: &CarpClient,
    study_id: &str,
    request: &SummaryExportRequest,
) -> ApiResult<()> {
    client
        .post_ok(
            &format!("/api/studies/{study_id}/exports/summaries"),
            request,
        )
        .await
}

pub async fn delete(client: &CarpClient, study_id: &str, export_id: &str) -> ApiResult<()> {
    client
        .delete_ok(&format!("/api/studies/{study_id}/exports/{export_id}"))
        .await
}

/// Path of the archive for a finished export.
pub fn download_path(study_id: &str, export_id: &str) -> String {
    format!("/api/studies/{study_id}/exports/{export_id}")
}
