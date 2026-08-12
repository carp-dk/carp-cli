// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Participant roles, and the data expected of them.
//!
//! A protocol names the *roles* people play in it. Most studies have one
//! (`Participant`); the family study has three - father, mother and child -
//! each carrying a phone and answering different surveys.
//!
//! Each role can be asked for data at enrolment through
//! [`ExpectedParticipantData`]: a name, a sex, a signed informed consent. The
//! informed consent one matters most, because without it CAWS has nothing to
//! record consent against.

use serde::{Deserialize, Serialize};

use crate::node::UnknownNode;

/// A part someone plays in a study.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticipantRole {
    /// The name the protocol refers to the role by, e.g. `Participant`.
    pub role: String,
    /// An optional role need not be filled for a deployment to run - the
    /// family study still works with no child enrolled.
    #[serde(default)]
    pub is_optional: bool,
}

impl ParticipantRole {
    /// A required role.
    pub fn new(role: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            is_optional: false,
        }
    }
}

/// A piece of data a participant is asked for, and who is asked.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpectedParticipantData {
    pub attribute: ParticipantAttribute,
    pub assigned_to: AssignedTo,
}

impl ExpectedParticipantData {
    /// Ask everyone in `roles` for `input_data_type`.
    pub fn for_roles(input_data_type: impl Into<String>, roles: Vec<String>) -> Self {
        Self {
            attribute: ParticipantAttribute::new(input_data_type),
            assigned_to: AssignedTo::Known(KnownAssignedTo::Roles {
                role_names: roles,
            }),
        }
    }

    /// Ask every role in the protocol for `input_data_type`.
    pub fn for_everyone(input_data_type: impl Into<String>) -> Self {
        Self {
            attribute: ParticipantAttribute::new(input_data_type),
            assigned_to: AssignedTo::Known(KnownAssignedTo::All {}),
        }
    }

    /// The input type asked for, e.g. `dk.cachet.carp.input.sex`.
    pub fn input_data_type(&self) -> &str {
        match &self.attribute {
            ParticipantAttribute::Known(KnownParticipantAttribute::Default {
                input_data_type,
            }) => input_data_type,
            ParticipantAttribute::Unknown(node) => node
                .field("inputDataType")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_else(|| node.short_type()),
        }
    }
}

/// What is asked for.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParticipantAttribute {
    Known(KnownParticipantAttribute),
    /// An attribute class this version does not model. See [`crate::node`].
    Unknown(UnknownNode),
}

/// The attribute classes this crate models.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "__type", rename_all_fields = "camelCase")]
pub enum KnownParticipantAttribute {
    /// An input type CARP already knows how to render a form for.
    ///
    /// The valid type names come from CAWS and CARP core rather than from this
    /// crate; `carp-catalog` discovers the ones in use upstream.
    #[serde(
        rename = "dk.cachet.carp.common.application.users.ParticipantAttribute.DefaultParticipantAttribute"
    )]
    Default { input_data_type: String },
}

impl ParticipantAttribute {
    pub fn new(input_data_type: impl Into<String>) -> Self {
        Self::Known(KnownParticipantAttribute::Default {
            input_data_type: input_data_type.into(),
        })
    }
}

/// Who is asked.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AssignedTo {
    Known(KnownAssignedTo),
    /// An assignment class this version does not model. See [`crate::node`].
    Unknown(UnknownNode),
}

/// The assignment classes this crate models.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "__type", rename_all_fields = "camelCase")]
pub enum KnownAssignedTo {
    /// Everyone in the study, whatever their role.
    #[serde(rename = "dk.cachet.carp.common.application.users.AssignedTo.All")]
    All {},

    /// Only the roles named.
    #[serde(rename = "dk.cachet.carp.common.application.users.AssignedTo.Roles")]
    Roles { role_names: Vec<String> },
}

impl AssignedTo {
    /// The roles this applies to, or `None` when it applies to everyone.
    pub fn role_names(&self) -> Option<&[String]> {
        match self {
            Self::Known(KnownAssignedTo::Roles { role_names }) => Some(role_names),
            _ => None,
        }
    }

    /// A phrase for the editor's list.
    pub fn label(&self) -> String {
        match self {
            Self::Known(KnownAssignedTo::All {}) => "everyone".to_owned(),
            Self::Known(KnownAssignedTo::Roles { role_names }) => role_names.join(", "),
            Self::Unknown(node) => node.short_type().to_owned(),
        }
    }

    /// Rename a role wherever this assignment mentions it.
    pub fn rename_role(&mut self, from: &str, to: &str) {
        if let Self::Known(KnownAssignedTo::Roles { role_names }) = self {
            for name in role_names.iter_mut() {
                if name == from {
                    to.clone_into(name);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
