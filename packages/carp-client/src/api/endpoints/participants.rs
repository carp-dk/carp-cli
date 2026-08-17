// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Participant endpoints.

use crate::api::client::CarpClient;
use crate::api::error::ApiResult;
use crate::api::models::{ParticipantPage, ParticipantQuery};

/// One page of the participant-centred view of a study.
///
/// `POST /api/studies/{study-id}/participants/accounts`
pub async fn query(
    client: &CarpClient,
    study_id: &str,
    query: &ParticipantQuery,
) -> ApiResult<ParticipantPage> {
    client
        .post_json(
            &format!("/api/studies/{study_id}/participants/accounts"),
            query,
        )
        .await
}
