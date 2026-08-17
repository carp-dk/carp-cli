// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Study models (`study-controller`).

use serde::{Deserialize, Serialize};

use crate::api::models::common::{CarpInstant, CarpUuid};

/// One row of `GET /api/studies/studies-overview`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct StudyOverview {
    pub study_id: CarpUuid,
    pub name: String,
    pub description: Option<String>,
    pub created_by: Option<String>,
    pub created_on: Option<CarpInstant>,
    pub study_protocol_id: Option<CarpUuid>,
    pub can_set_invitation: bool,
    pub can_set_study_protocol: bool,
    pub can_deploy_to_participants: bool,
}

impl StudyOverview {
    /// Short lifecycle label derived from the capability flags the API returns:
    /// a study that can still be given a protocol has not gone live yet.
    pub fn stage(&self) -> &'static str {
        match (
            self.study_protocol_id.is_some(),
            self.can_deploy_to_participants,
        ) {
            (_, true) => "live",
            (true, false) => "configured",
            (false, false) => "draft",
        }
    }

    pub fn description_line(&self) -> &str {
        self.description
            .as_deref()
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .unwrap_or("-")
    }

    /// Case-insensitive match used by the study filter.
    pub fn matches(&self, needle: &str) -> bool {
        let needle = needle.to_lowercase();
        self.name.to_lowercase().contains(&needle)
            || self.study_id.as_str().to_lowercase().contains(&needle)
            || self
                .description
                .as_deref()
                .is_some_and(|text| text.to_lowercase().contains(&needle))
            || self
                .created_by
                .as_deref()
                .is_some_and(|text| text.to_lowercase().contains(&needle))
    }
}

/// One row of `GET /api/studies/{study-id}/inactive_deployments`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct InactiveDeployment {
    pub deployment_id: CarpUuid,
    pub date_of_last_data_upload: Option<CarpInstant>,
}
