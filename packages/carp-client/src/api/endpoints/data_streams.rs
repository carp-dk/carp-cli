// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Uploaded-data endpoints (`data-stream-controller`,
//! `study-deployment-controller`).

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::api::client::CarpClient;
use crate::api::error::ApiResult;
use crate::api::models::protocol::{API_VERSION, ApiVersion};
use crate::api::models::{DataStreamBatch, DataStreamId, DataStreamSummary, DeploymentStatistics};

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

/// The measurements one stream holds within a window of wall-clock time.
///
/// `POST /api/data-stream-service/query-by-time`
///
/// This is the endpoint to reach for when the question is "what did this study
/// record last week": the core `getDataStream` selects a range of sequence ids,
/// which nobody knows off-hand, whereas `from` and `to` are what a researcher
/// actually has. Both ends are inclusive, and both are compared against the
/// measurement's own `updated_at`.
pub async fn query_by_time(
    client: &CarpClient,
    stream: &DataStreamId,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> ApiResult<DataStreamBatch> {
    client
        .post_json_with_query(
            "/api/data-stream-service/query-by-time",
            stream,
            &[("from", from.to_rfc3339()), ("to", to.to_rfc3339())],
        )
        .await
}

/// A command sent to `/api/data-stream-service`.
///
/// Only the read is modelled. Opening, appending to and closing streams is the
/// study app's job — this client reads what a study collected, and an
/// `AppendToDataStreams` sent by hand would be writing participant data.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "__type")]
enum DataStreamRequest {
    #[serde(
        rename = "dk.cachet.carp.data.infrastructure.DataStreamServiceRequest.GetDataStream",
        rename_all = "camelCase"
    )]
    GetDataStream {
        data_stream: DataStreamId,
        from_sequence_id: i64,
        #[serde(skip_serializing_if = "Option::is_none")]
        to_sequence_id_inclusive: Option<i64>,
        api_version: ApiVersion,
    },
}

/// The measurements one stream holds within a range of sequence ids.
///
/// `POST /api/data-stream-service`, the core `getDataStream`. Prefer
/// [`query_by_time`] unless the sequence numbering is what you have — this is
/// the way to page through a stream exhaustively, since ids are dense and a
/// batch reports the ones it returned.
pub async fn get_data_stream(
    client: &CarpClient,
    stream: &DataStreamId,
    from_sequence_id: i64,
    to_sequence_id_inclusive: Option<i64>,
) -> ApiResult<DataStreamBatch> {
    client
        .post_json(
            "/api/data-stream-service",
            &DataStreamRequest::GetDataStream {
                data_stream: stream.clone(),
                from_sequence_id,
                to_sequence_id_inclusive,
                api_version: API_VERSION,
            },
        )
        .await
}
