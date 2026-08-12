// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Account endpoints (`account-controller`).

use std::collections::HashMap;

use crate::api::client::CarpClient;
use crate::api::error::ApiResult;
use crate::api::models::Account;

/// OAuth redirect URIs the deployment has registered, keyed by client.
///
/// These are the addresses Keycloak is allowed to return browsers to, which
/// makes them the deployment's own record of where its web clients live.
pub async fn redirect_uris(client: &CarpClient) -> ApiResult<HashMap<String, Vec<String>>> {
    client.get_json("/api/accounts/redirect-uris", &[]).await
}

/// A single account by id.
pub async fn info(client: &CarpClient, account_id: &str) -> ApiResult<Account> {
    client
        .get_json(&format!("/api/accounts/{account_id}"), &[])
        .await
}
