// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Whether the participant roles and the data expected of them agree.

use std::collections::HashSet;

use super::super::Diagnostic;
use crate::protocol::StudyProtocol;

/// Participant roles are unique, and expected data is assigned to roles that
/// exist.
pub fn participants(protocol: &StudyProtocol, out: &mut Vec<Diagnostic>) {
    let mut seen: HashSet<&str> = HashSet::new();
    for role in &protocol.participant_roles {
        if role.role.trim().is_empty() {
            out.push(Diagnostic::error("participant roles", "a role has no name"));
        } else if !seen.insert(role.role.as_str()) {
            out.push(Diagnostic::error(
                format!("participant role {:?}", role.role),
                "is defined twice",
            ));
        }
    }

    for expected in &protocol.expected_participant_data {
        let Some(roles) = expected.assigned_to.role_names() else {
            continue;
        };
        for role in roles {
            if !seen.contains(role.as_str()) {
                out.push(
                    Diagnostic::error(
                        format!("expected data {:?}", expected.input_data_type()),
                        format!("is assigned to {role:?}, which is not a participant role"),
                    )
                    .with_hint("add the role, or reassign the expected data"),
                );
            }
        }
    }

    // Without an informed consent among the expected data, CAWS has nothing to
    // record consent against, and the study app never asks for it.
    let has_consent = protocol
        .expected_participant_data
        .iter()
        .any(|expected| expected.input_data_type().ends_with("informed_consent"));
    if !protocol.participant_roles.is_empty() && !has_consent {
        out.push(
            Diagnostic::warning("expected data", "no informed consent is expected")
                .with_hint("without it nothing is uploaded as a signed consent"),
        );
    }
}
