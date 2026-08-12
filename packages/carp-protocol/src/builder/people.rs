// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Adding, renaming and removing participant roles.
//!
//! See [`super`] for why every mutation goes through this module.

use crate::participant::ParticipantRole;
use crate::protocol::StudyProtocol;

use super::unique_name;

/// Add a participant role, returning the name it settled on.
pub fn add_participant_role(protocol: &mut StudyProtocol, name: &str) -> String {
    let taken: Vec<String> = protocol
        .participant_roles
        .iter()
        .map(|role| role.role.clone())
        .collect();
    let name = unique_name(name, &taken);
    protocol
        .participant_roles
        .push(ParticipantRole::new(name.clone()));
    name
}

/// Rename a participant role and every expected-data entry assigned to it.
pub fn rename_participant_role(protocol: &mut StudyProtocol, from: &str, to: &str) -> bool {
    if from == to {
        return true;
    }
    let taken = protocol
        .participant_roles
        .iter()
        .any(|role| role.role == to);
    if to.trim().is_empty() || taken {
        return false;
    }

    let Some(role) = protocol
        .participant_roles
        .iter_mut()
        .find(|role| role.role == from)
    else {
        return false;
    };
    to.clone_into(&mut role.role);

    for expected in &mut protocol.expected_participant_data {
        expected.assigned_to.rename_role(from, to);
    }
    for devices in std::mem::take(&mut protocol.assigned_devices) {
        let (role, assigned) = devices;
        let key = if role == from { to.to_owned() } else { role };
        protocol.assigned_devices.insert(key, assigned);
    }
    true
}

/// Remove a participant role and every expected-data entry that only applied
/// to it.
pub fn remove_participant_role(protocol: &mut StudyProtocol, name: &str) {
    protocol.participant_roles.retain(|role| role.role != name);
    protocol.assigned_devices.remove(name);

    protocol.expected_participant_data.retain_mut(|expected| {
        let Some(roles) = expected.assigned_to.role_names() else {
            // Assigned to everyone; unaffected by one role going.
            return true;
        };
        let remaining: Vec<String> = roles
            .iter()
            .filter(|role| role.as_str() != name)
            .cloned()
            .collect();
        if remaining.is_empty() {
            return false;
        }
        expected.assigned_to =
            crate::participant::AssignedTo::Known(crate::participant::KnownAssignedTo::Roles {
                role_names: remaining,
            });
        true
    });
}
