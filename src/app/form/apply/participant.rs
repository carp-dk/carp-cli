// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Applying the participant-role and expected-data forms.

use carp_protocol::StudyProtocol;
use carp_protocol::builder;
use carp_protocol::participant::{AssignedTo, KnownAssignedTo, ParticipantAttribute};

use crate::app::form::Form;

use super::Applied;

/// Write a participant-role form back, renaming through [`builder`] so the
/// expected data assigned to the role moves with it.
pub fn apply_role(protocol: &mut StudyProtocol, form: &Form, role: &str) -> Applied {
    if !protocol
        .participant_roles
        .iter()
        .any(|existing| existing.role == role)
    {
        return Applied::Vanished;
    }

    let new_role = form.text("role");
    if new_role.trim().is_empty() {
        return Applied::Refused("a role needs a name".to_owned());
    }
    if new_role != role && !builder::rename_participant_role(protocol, role, &new_role) {
        return Applied::Refused(format!("a role called {new_role:?} already exists"));
    }

    let optional = form.flag("is_optional");
    if let Some(existing) = protocol
        .participant_roles
        .iter_mut()
        .find(|existing| existing.role == new_role)
    {
        existing.is_optional = optional;
    }
    Applied::Changed
}

/// Write an expected-participant-data form back.
///
/// An empty "asked of" means everyone, which is a different value on the wire
/// (`AssignedTo.All`) rather than a role list that happens to be empty.
pub fn apply_expected(protocol: &mut StudyProtocol, form: &Form, index: usize) -> Applied {
    let Some(expected) = protocol.expected_participant_data.get_mut(index) else {
        return Applied::Vanished;
    };

    let input_data_type = form.text("input_data_type");
    if input_data_type.trim().is_empty() {
        return Applied::Refused("choose what the participant is asked for".to_owned());
    }
    expected.attribute = ParticipantAttribute::new(input_data_type);

    let role = form.text("assigned_to");
    expected.assigned_to = if role.is_empty() {
        AssignedTo::Known(KnownAssignedTo::All {})
    } else {
        AssignedTo::Known(KnownAssignedTo::Roles {
            role_names: vec![role],
        })
    };

    Applied::Changed
}
