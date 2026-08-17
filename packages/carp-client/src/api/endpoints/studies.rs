// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Study endpoints (`study-controller`).

use chrono::{DateTime, Utc};

use crate::api::client::CarpClient;
use crate::api::error::ApiResult;
use crate::api::models::{Account, InactiveDeployment, ParticipantGroupStatus, StudyOverview};

/// Studies the signed-in account can see.
pub async fn list(client: &CarpClient) -> ApiResult<Vec<StudyOverview>> {
    client.get_json("/api/studies/studies-overview", &[]).await
}

/// Researchers attached to a study.
pub async fn researchers(client: &CarpClient, study_id: &str) -> ApiResult<Vec<Account>> {
    client
        .get_json(&format!("/api/studies/{study_id}/researchers"), &[])
        .await
}

/// Research assistants attached to a study.
pub async fn research_assistants(client: &CarpClient, study_id: &str) -> ApiResult<Vec<Account>> {
    client
        .get_json(&format!("/api/studies/{study_id}/research-assistants"), &[])
        .await
}

/// Participant groups of a study and the state of their deployments.
///
/// The endpoint is documented as returning a string, but deployments answer
/// with the group document; both are accepted.
pub async fn participant_group_status(
    client: &CarpClient,
    study_id: &str,
) -> ApiResult<ParticipantGroupStatus> {
    let body = client
        .get_text(&format!("/api/studies/{study_id}/participantGroup/status"))
        .await?;
    Ok(serde_json::from_str(&body).unwrap_or_else(|_| ParticipantGroupStatus::from_label(body)))
}

/// Deployments that have not uploaded data since `last_update`.
pub async fn inactive_deployments(
    client: &CarpClient,
    study_id: &str,
    last_update: DateTime<Utc>,
    limit: u32,
) -> ApiResult<Vec<InactiveDeployment>> {
    client
        .get_json(
            &format!("/api/studies/{study_id}/inactive_deployments"),
            &[
                ("last_update", last_update.to_rfc3339()),
                ("offset", "0".to_owned()),
                ("limit", limit.to_string()),
            ],
        )
        .await
}
