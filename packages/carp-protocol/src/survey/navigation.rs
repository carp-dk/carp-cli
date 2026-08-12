// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Branching inside a survey.
//!
//! An [`super::KnownRpTask::Navigable`] survey attaches rules to steps: after the
//! participant answers *this* step, jump to *that* one depending on what they
//! answered. That is how "do you smoke?" skips the three follow-up questions
//! when the answer is no.
//!
//! The rule is keyed by the answer's *value* rendered as a string - the
//! `value` of the [`crate::survey::RpChoice`] they picked, not its label - so
//! renumbering choices silently rewires the survey. [`mod@crate::validate`] checks
//! that every target exists, which catches the commoner mistake of renaming a
//! step and leaving the rule behind.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::node::UnknownNode;

/// What happens after a step is answered.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RpStepNavigationRule {
    Known(KnownNavigationRule),
    /// A rule type this version does not model. See [`crate::node`].
    Unknown(UnknownNode),
}

/// The navigation rules this crate models.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "__type", rename_all_fields = "camelCase")]
pub enum KnownNavigationRule {
    /// Jump to a step chosen by the answer.
    #[serde(rename = "RPStepJumpRule")]
    Jump {
        /// Answer value (as a string) -> identifier of the step to go to.
        answer_map: BTreeMap<String, String>,
    },
}

impl RpStepNavigationRule {
    /// A jump rule from answer values to step identifiers.
    pub fn jump(answer_map: BTreeMap<String, String>) -> Self {
        Self::Known(KnownNavigationRule::Jump { answer_map })
    }

    /// The step identifiers this rule can send a participant to.
    pub fn destinations(&self) -> Vec<&str> {
        match self {
            Self::Known(KnownNavigationRule::Jump { answer_map }) => {
                answer_map.values().map(String::as_str).collect()
            }
            Self::Unknown(_) => Vec::new(),
        }
    }

    /// Point every branch that went to `from` at `to` instead.
    pub fn rename_destination(&mut self, from: &str, to: &str) {
        if let Self::Known(KnownNavigationRule::Jump { answer_map }) = self {
            for destination in answer_map.values_mut() {
                if destination == from {
                    to.clone_into(destination);
                }
            }
        }
    }

    /// A one-line summary for the editor.
    pub fn label(&self) -> String {
        match self {
            Self::Known(KnownNavigationRule::Jump { answer_map }) => {
                format!("{} branches", answer_map.len())
            }
            Self::Unknown(node) => node.short_type().to_owned(),
        }
    }
}

#[cfg(test)]
mod tests;
