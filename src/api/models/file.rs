// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Study file models (`file-controller`).

use serde::{Deserialize, Serialize};

use crate::api::models::common::CarpInstant;

/// One row of `GET /api/studies/{study-id}/files`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct StudyFile {
    pub id: i32,
    pub file_name: String,
    pub original_name: String,
    pub relative_path: String,
    pub study_id: String,
    pub owner_id: Option<String>,
    pub deployment_id: Option<String>,
    pub created_by: Option<String>,
    pub created_at: Option<CarpInstant>,
    pub metadata: Option<serde_json::Value>,
}

impl StudyFile {
    /// Name to save the download as.
    pub fn download_name(&self) -> &str {
        if self.original_name.is_empty() {
            &self.file_name
        } else {
            &self.original_name
        }
    }

    /// Deployment this file belongs to, shortened for table display.
    pub fn deployment_label(&self) -> &str {
        self.deployment_id
            .as_deref()
            .map(|id| id.split('-').next().unwrap_or(id))
            .unwrap_or("-")
    }
}
