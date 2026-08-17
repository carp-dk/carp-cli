// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Export models (`export-controller`). An export is a server-side job that
//! packages study data into a downloadable archive.

use serde::{Deserialize, Deserializer, Serialize};

use crate::api::models::common::{CarpInstant, CarpUuid};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExportStatus {
    #[default]
    Unknown,
    InProgress,
    Available,
    Error,
    Expired,
}

impl ExportStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::InProgress => "in progress",
            Self::Available => "available",
            Self::Error => "error",
            Self::Expired => "expired",
        }
    }

    /// Only available exports can be downloaded.
    pub fn is_downloadable(self) -> bool {
        self == Self::Available
    }

    /// While an export is being produced the list is worth polling.
    pub fn is_pending(self) -> bool {
        self == Self::InProgress
    }
}

impl<'de> Deserialize<'de> for ExportStatus {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Ok(match value.as_str() {
            "IN_PROGRESS" => Self::InProgress,
            "AVAILABLE" => Self::Available,
            "ERROR" => Self::Error,
            "EXPIRED" => Self::Expired,
            _ => Self::Unknown,
        })
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExportKind {
    #[default]
    Unknown,
    StudyData,
    DeploymentData,
    AnonymousParticipants,
}

impl ExportKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::StudyData => "study data",
            Self::DeploymentData => "deployment data",
            Self::AnonymousParticipants => "anonymous participants",
        }
    }
}

impl<'de> Deserialize<'de> for ExportKind {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Ok(match value.as_str() {
            "STUDY_DATA" => Self::StudyData,
            "DEPLOYMENT_DATA" => Self::DeploymentData,
            "ANONYMOUS_PARTICIPANTS" => Self::AnonymousParticipants,
            _ => Self::Unknown,
        })
    }
}

/// One row of `GET /api/studies/{study-id}/exports`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Export {
    pub id: String,
    pub file_name: String,
    pub relative_path: String,
    pub study_id: String,
    pub status: ExportStatus,
    #[serde(rename = "type")]
    pub kind: ExportKind,
    pub created_by: Option<String>,
    pub created_at: Option<CarpInstant>,
    pub updated_at: Option<CarpInstant>,
}

impl Export {
    /// Name to show and to save the archive as.
    ///
    /// A freshly requested export has no `fileName` until the server has
    /// packaged it, so fall back to the path it will land on and finally to
    /// the export id - a row must never render blank.
    pub fn display_name(&self) -> String {
        let from_field = self.file_name.trim();
        if !from_field.is_empty() {
            return from_field.to_owned();
        }
        let from_path = self
            .relative_path
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or_default()
            .trim();
        if !from_path.is_empty() {
            return from_path.to_owned();
        }
        match self.status {
            ExportStatus::InProgress => format!("{} (being packaged)", self.short_id()),
            _ => self.short_id().to_owned(),
        }
    }

    pub fn short_id(&self) -> &str {
        self.id.split('-').next().unwrap_or(&self.id)
    }
}

/// Request body for `POST /api/studies/{study-id}/exports/summaries`.
///
/// With no deployment ids the server exports the whole study.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SummaryExportRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment_ids: Option<Vec<CarpUuid>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_deployments_only: Option<bool>,
}
