// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Models for the protocol service.
//!
//! CARP's protocol service is a *command* endpoint: every operation is a POST
//! to `/api/protocol-service` whose body is one of a set of request objects,
//! distinguished by a `__type`. That is why these look like commands rather
//! than like resources.
//!
//! Each request also carries an `apiVersion`, which CAWS uses to decide how to
//! read the payload. It is not the protocol's version and not this tool's; see
//! [`API_VERSION`].

use serde::{Deserialize, Serialize};

/// Version of the protocol service's own API that these requests speak.
///
/// Distinct from [`carp_protocol::StudyProtocol::version`], which counts
/// revisions of one protocol, and from the CLI's own version. It changes only
/// when CARP changes the shape of these commands.
pub const API_VERSION: ApiVersion = ApiVersion { major: 1, minor: 3 };

/// The `apiVersion` field every protocol-service command carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiVersion {
    pub major: u32,
    pub minor: u32,
}

/// A command sent to `/api/protocol-service`.
///
/// Serialised with a `__type` naming the Kotlin request class, which is how
/// CAWS dispatches it.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "__type")]
pub enum ProtocolRequest {
    /// Store a protocol that CAWS has not seen before.
    #[serde(
        rename = "dk.cachet.carp.protocols.infrastructure.ProtocolServiceRequest.Add",
        rename_all = "camelCase"
    )]
    Add {
        protocol: carp_protocol::StudyProtocol,
        /// The label this revision is filed under. CAWS rejects a tag already
        /// used for the same protocol.
        version_tag: String,
        api_version: ApiVersion,
    },

    /// Store a new revision of a protocol CAWS already holds.
    ///
    /// The protocol's id decides which one; that is why
    /// [`carp_protocol::version::fork`] exists for the case where a copy
    /// should become its own protocol instead.
    #[serde(
        rename = "dk.cachet.carp.protocols.infrastructure.ProtocolServiceRequest.AddVersion",
        rename_all = "camelCase"
    )]
    AddVersion {
        protocol: carp_protocol::StudyProtocol,
        version_tag: String,
        api_version: ApiVersion,
    },

    /// The revisions CAWS holds for a protocol.
    #[serde(
        rename = "dk.cachet.carp.protocols.infrastructure.ProtocolServiceRequest.GetVersionHistoryFor",
        rename_all = "camelCase"
    )]
    GetVersionHistoryFor {
        protocol_id: String,
        api_version: ApiVersion,
    },
}

impl ProtocolRequest {
    /// Store `protocol` under `version_tag`.
    ///
    /// `first` selects between `Add` and `AddVersion`: CAWS rejects an `Add`
    /// for a protocol it already holds, and an `AddVersion` for one it does
    /// not, so the caller has to know which it is doing.
    pub fn store(protocol: carp_protocol::StudyProtocol, version_tag: String, first: bool) -> Self {
        if first {
            Self::Add {
                protocol,
                version_tag,
                api_version: API_VERSION,
            }
        } else {
            Self::AddVersion {
                protocol,
                version_tag,
                api_version: API_VERSION,
            }
        }
    }

    pub fn version_history(protocol_id: String) -> Self {
        Self::GetVersionHistoryFor {
            protocol_id,
            api_version: API_VERSION,
        }
    }
}

/// One revision CAWS holds, as `GetVersionHistoryFor` returns it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ProtocolVersion {
    pub tag: String,
    pub date: String,
}

/// A protocol as `/api/protocols-overview` lists it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolOverview {
    #[serde(default)]
    pub owner_name: Option<String>,
    #[serde(default)]
    pub version_tag: String,
    #[serde(default)]
    pub last_version_created_date: Option<String>,
    /// The protocol itself. Kept as raw JSON: the overview is a listing, and
    /// parsing every protocol in it to show a name would be wasteful.
    #[serde(default)]
    pub snapshot: Option<serde_json::Value>,
}

impl ProtocolOverview {
    /// The protocol's name, from the snapshot.
    pub fn name(&self) -> &str {
        self.snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.get("name"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unnamed protocol")
    }

    pub fn id(&self) -> Option<&str> {
        self.snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.get("id"))
            .and_then(serde_json::Value::as_str)
    }
}

#[cfg(test)]
mod tests;
