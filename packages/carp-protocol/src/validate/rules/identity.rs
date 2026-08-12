// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Whether the protocol says who and what it is.


use super::super::Diagnostic;
use crate::protocol::StudyProtocol;

pub fn identity(protocol: &StudyProtocol, out: &mut Vec<Diagnostic>) {
    if protocol.name.trim().is_empty() {
        out.push(
            Diagnostic::error("protocol", "has no name")
                .with_hint("the name identifies the protocol in CAWS"),
        );
    }
    if uuid::Uuid::parse_str(&protocol.id).is_err() {
        out.push(
            Diagnostic::error("protocol", format!("id {:?} is not a UUID", protocol.id))
                .with_hint("CAWS rejects a protocol whose id is not a UUID"),
        );
    }
    // A warning rather than an error: CAWS requires a UUID here, but a
    // protocol kept in source control legitimately carries a placeholder that
    // the upload step substitutes - the ICAT study writes
    // `to_be_set_on_upload`. Uploading is where it has to be a UUID, and
    // `crate::version::UploadCheck` enforces it there.
    if uuid::Uuid::parse_str(&protocol.owner_id).is_err() {
        out.push(
            Diagnostic::warning(
                "protocol",
                format!("owner id {:?} is not a UUID", protocol.owner_id),
            )
            .with_hint("CAWS requires a UUID; it is replaced with the uploader's id"),
        );
    }
    if chrono::DateTime::parse_from_rfc3339(&protocol.created_on).is_err() {
        out.push(Diagnostic::error(
            "protocol",
            format!("created-on {:?} is not an ISO-8601 instant", protocol.created_on),
        ));
    }
}
