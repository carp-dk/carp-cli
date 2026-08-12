// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Uploaded-data endpoints (`data-stream-controller`,
//! `study-deployment-controller`).

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::api::client::CarpClient;
use crate::api::error::ApiResult;
use crate::api::models::{DataStreamSummary, DeploymentStatistics};

/// Arguments of `GET /api/data-stream-service/summary`.
///
/// `scope` and `kind` are free-form strings in the OpenAPI document; the
/// accepted values are defined server side, so they are passed straight
/// through rather than modelled as enums here.
#[derive(Debug, Clone)]
pub struct SummaryQuery {
    pub study_id: String,
    pub deployment_id: Option<String>,
    pub participant_id: Option<String>,
    pub scope: String,
    pub kind: String,
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
}

/// Upload volume over time for a study, deployment or participant.
pub async fn summary(client: &CarpClient, query: &SummaryQuery) -> ApiResult<DataStreamSummary> {
    let mut params = vec![
        ("studyId", query.study_id.clone()),
        ("scope", query.scope.clone()),
        ("type", query.kind.clone()),
        ("from", query.from.to_rfc3339()),
        ("to", query.to.to_rfc3339()),
    ];
    if let Some(deployment_id) = &query.deployment_id {
        params.push(("deploymentId", deployment_id.clone()));
    }
    if let Some(participant_id) = &query.participant_id {
        params.push(("participantId", participant_id.clone()));
    }
    client
        .get_json("/api/data-stream-service/summary", &params)
        .await
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct StatisticsRequest {
    deployment_ids: Vec<String>,
}

/// Upload counts per deployment.
pub async fn deployment_statistics(
    client: &CarpClient,
    deployment_ids: Vec<String>,
) -> ApiResult<DeploymentStatistics> {
    client
        .post_json(
            "/api/deployment-service/statistics",
            &StatisticsRequest { deployment_ids },
        )
        .await
}
