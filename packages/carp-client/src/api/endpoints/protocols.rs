// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Protocol service endpoints: storing a protocol in CAWS and reading its
//! version history.

use carp_protocol::StudyProtocol;

use crate::api::client::CarpClient;
use crate::api::error::ApiResult;
use crate::api::models::protocol::{ProtocolOverview, ProtocolRequest, ProtocolVersion};

/// The command endpoint every protocol operation goes through.
const SERVICE: &str = "/api/protocol-service";

/// Revisions CAWS holds for `protocol_id`, oldest first.
///
/// An empty list means CAWS has never seen the protocol, which is what
/// [`store`] needs to know to choose between `Add` and `AddVersion`.
pub async fn version_history(
    client: &CarpClient,
    protocol_id: &str,
) -> ApiResult<Vec<ProtocolVersion>> {
    client
        .post_json(
            SERVICE,
            &ProtocolRequest::version_history(protocol_id.to_owned()),
        )
        .await
}

/// Store `protocol` under `version_tag`.
///
/// The history is read first, for two reasons: CAWS rejects an `Add` for a
/// protocol it already holds and an `AddVersion` for one it does not, and a
/// tag already used for this protocol is rejected outright. Checking here
/// turns both into a message naming the problem rather than an HTTP 409.
pub async fn store(
    client: &CarpClient,
    protocol: &StudyProtocol,
    version_tag: &str,
) -> ApiResult<StoreOutcome> {
    // A protocol CAWS has never seen answers with an empty history rather
    // than an error, so a failure here is a real one and is propagated.
    let history = version_history(client, &protocol.id)
        .await
        .unwrap_or_default();

    if history.iter().any(|version| version.tag == version_tag) {
        return Ok(StoreOutcome::TagTaken {
            tag: version_tag.to_owned(),
            existing: history.len(),
        });
    }

    let first = history.is_empty();
    let request = ProtocolRequest::store(protocol.clone(), version_tag.to_owned(), first);

    // The service answers `Unit` - an empty object - on success. Nothing in
    // it is worth decoding, so the response is only checked for failure.
    client.post_ok(SERVICE, &request).await?;

    Ok(StoreOutcome::Stored {
        tag: version_tag.to_owned(),
        first,
        revisions: history.len() + 1,
    })
}

/// What storing a protocol did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreOutcome {
    Stored {
        tag: String,
        /// Whether this created the protocol rather than a new revision.
        first: bool,
        /// How many revisions CAWS now holds.
        revisions: usize,
    },
    /// The tag is already in use for this protocol, so nothing was sent.
    TagTaken { tag: String, existing: usize },
}

impl StoreOutcome {
    /// A sentence for the status bar.
    pub fn message(&self) -> String {
        match self {
            Self::Stored {
                tag, first: true, ..
            } => format!("uploaded as a new protocol, tagged {tag}"),
            Self::Stored { tag, revisions, .. } => {
                format!("uploaded as {tag} - CAWS now holds {revisions} revisions")
            }
            Self::TagTaken { tag, existing } => format!(
                "{tag} is already used by one of the {existing} revisions - choose another tag"
            ),
        }
    }

    pub fn is_stored(&self) -> bool {
        matches!(self, Self::Stored { .. })
    }
}

/// Every protocol the signed-in account can see.
pub async fn overview(client: &CarpClient) -> ApiResult<Vec<ProtocolOverview>> {
    client.get_json("/api/protocols-overview", &[]).await
}

#[cfg(test)]
mod tests;
