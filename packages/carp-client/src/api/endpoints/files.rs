// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Study file endpoints (`file-controller`).

use crate::api::client::CarpClient;
use crate::api::error::ApiResult;
use crate::api::models::StudyFile;

/// Files uploaded for a study, optionally filtered by the server-side query
/// expression the API accepts.
pub async fn list(
    client: &CarpClient,
    study_id: &str,
    query: Option<&str>,
) -> ApiResult<Vec<StudyFile>> {
    let params = query
        .filter(|value| !value.is_empty())
        .map(|value| vec![("query", value.to_owned())])
        .unwrap_or_default();
    client
        .get_json(&format!("/api/studies/{study_id}/files"), &params)
        .await
}

pub async fn delete(client: &CarpClient, study_id: &str, file_id: i32) -> ApiResult<()> {
    client
        .delete_ok(&format!("/api/studies/{study_id}/files/{file_id}"))
        .await
}

/// Path of a file's contents.
pub fn download_path(study_id: &str, file_id: i32) -> String {
    format!("/api/studies/{study_id}/files/{file_id}/download")
}
