// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Surveys, as the Research Package models them.
//!
//! An `RPAppTask` carries one [`RpTask`]: an identifier, a list of
//! [`step`]s and, for a navigable survey, the branching rules between them.
//! The `RP` prefix throughout is Research Package's, kept here so the type
//! names match the JSON and the Dart originals.
//!
//! # Layout
//!
//! - [`step`] - the pages, from instructions to cognitive activities
//! - [`answer`] - how a question is answered
//! - [`choice`] - the options of a choice question
//! - [`navigation`] - branching between steps
//!
//! Nothing here is a `Vec<Step>` with a fixed shape: a form step nests further
//! steps inside itself, so the tree is genuinely recursive and the editor
//! walks it as one.

pub mod answer;
pub mod choice;
pub mod navigation;
pub mod step;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub use answer::{KnownAnswerFormat, RpAnswerFormat};
pub use choice::{RpChoice, RpImageChoice};
pub use navigation::{KnownNavigationRule, RpStepNavigationRule};
pub use step::{KnownStep, RpStep};

use crate::node::UnknownNode;

/// A survey carried by an `RPAppTask`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RpTask {
    Known(Box<KnownRpTask>),
    /// A survey type this version does not model. See [`crate::node`].
    Unknown(UnknownNode),
}

/// The survey types this crate models.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "__type", rename_all_fields = "camelCase")]
pub enum KnownRpTask {
    /// Steps shown one after another, start to finish.
    #[serde(rename = "RPOrderedTask")]
    Ordered {
        identifier: String,
        /// Whether the survey closes itself once finished, rather than
        /// waiting for the participant to dismiss it.
        #[serde(default = "yes")]
        close_after_finished: bool,
        #[serde(default)]
        steps: Vec<RpStep>,
    },

    /// Steps with branching between them. See [`navigation`].
    #[serde(rename = "RPNavigableOrderedTask")]
    Navigable {
        identifier: String,
        #[serde(default = "yes")]
        close_after_finished: bool,
        #[serde(default)]
        steps: Vec<RpStep>,
        /// Step identifier -> what happens after that step.
        #[serde(default)]
        step_navigation_rules: BTreeMap<String, RpStepNavigationRule>,
    },
}

fn yes() -> bool {
    true
}

impl Default for RpTask {
    fn default() -> Self {
        Self::ordered("survey")
    }
}

impl RpTask {
    /// An empty linear survey.
    pub fn ordered(identifier: impl Into<String>) -> Self {
        Self::Known(Box::new(KnownRpTask::Ordered {
            identifier: identifier.into(),
            close_after_finished: true,
            steps: Vec::new(),
        }))
    }

    /// An empty branching survey.
    pub fn navigable(identifier: impl Into<String>) -> Self {
        Self::Known(Box::new(KnownRpTask::Navigable {
            identifier: identifier.into(),
            close_after_finished: true,
            steps: Vec::new(),
            step_navigation_rules: BTreeMap::new(),
        }))
    }

    /// The survey's own identifier.
    pub fn identifier(&self) -> &str {
        match self {
            Self::Known(task) => match task.as_ref() {
                KnownRpTask::Ordered { identifier, .. }
                | KnownRpTask::Navigable { identifier, .. } => identifier,
            },
            Self::Unknown(node) => node
                .field("identifier")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default(),
        }
    }

    pub fn set_identifier(&mut self, value: impl Into<String>) {
        let value = value.into();
        match self {
            Self::Known(task) => match task.as_mut() {
                KnownRpTask::Ordered { identifier, .. }
                | KnownRpTask::Navigable { identifier, .. } => *identifier = value,
            },
            Self::Unknown(node) => {
                node.fields
                    .insert("identifier".to_owned(), serde_json::Value::String(value));
            }
        }
    }

    /// The pages, in order.
    pub fn steps(&self) -> &[RpStep] {
        match self {
            Self::Known(task) => match task.as_ref() {
                KnownRpTask::Ordered { steps, .. } | KnownRpTask::Navigable { steps, .. } => steps,
            },
            Self::Unknown(_) => &[],
        }
    }

    pub fn steps_mut(&mut self) -> Option<&mut Vec<RpStep>> {
        match self {
            Self::Known(task) => match task.as_mut() {
                KnownRpTask::Ordered { steps, .. } | KnownRpTask::Navigable { steps, .. } => {
                    Some(steps)
                }
            },
            Self::Unknown(_) => None,
        }
    }

    /// The branching rules, for a navigable survey.
    pub fn navigation_rules(&self) -> Option<&BTreeMap<String, RpStepNavigationRule>> {
        match self {
            Self::Known(task) => match task.as_ref() {
                KnownRpTask::Navigable {
                    step_navigation_rules,
                    ..
                } => Some(step_navigation_rules),
                KnownRpTask::Ordered { .. } => None,
            },
            Self::Unknown(_) => None,
        }
    }

    pub fn navigation_rules_mut(&mut self) -> Option<&mut BTreeMap<String, RpStepNavigationRule>> {
        match self {
            Self::Known(task) => match task.as_mut() {
                KnownRpTask::Navigable {
                    step_navigation_rules,
                    ..
                } => Some(step_navigation_rules),
                KnownRpTask::Ordered { .. } => None,
            },
            Self::Unknown(_) => None,
        }
    }

    pub fn type_label(&self) -> &str {
        match self {
            Self::Known(task) => match task.as_ref() {
                KnownRpTask::Ordered { .. } => "RPOrderedTask",
                KnownRpTask::Navigable { .. } => "RPNavigableOrderedTask",
            },
            Self::Unknown(node) => node.short_type(),
        }
    }

    /// Turn a linear survey into a branching one, or the reverse.
    ///
    /// Going back to linear discards the rules, because an ordered task has
    /// nowhere to keep them. The steps are untouched either way.
    pub fn set_navigable(&mut self, navigable: bool) {
        let Self::Known(task) = self else {
            return;
        };
        match (navigable, task.as_mut()) {
            (
                true,
                KnownRpTask::Ordered {
                    identifier,
                    close_after_finished,
                    steps,
                },
            ) => {
                **task = KnownRpTask::Navigable {
                    identifier: std::mem::take(identifier),
                    close_after_finished: *close_after_finished,
                    steps: std::mem::take(steps),
                    step_navigation_rules: BTreeMap::new(),
                };
            }
            (
                false,
                KnownRpTask::Navigable {
                    identifier,
                    close_after_finished,
                    steps,
                    ..
                },
            ) => {
                **task = KnownRpTask::Ordered {
                    identifier: std::mem::take(identifier),
                    close_after_finished: *close_after_finished,
                    steps: std::mem::take(steps),
                };
            }
            _ => {}
        }
    }

    /// Every step identifier in the survey, including those nested in forms.
    ///
    /// Navigation rules may target a nested question, so a flat walk is what
    /// validation needs rather than the top-level list.
    pub fn all_step_identifiers(&self) -> Vec<String> {
        fn walk(steps: &[RpStep], out: &mut Vec<String>) {
            for step in steps {
                out.push(step.identifier().to_owned());
                if let RpStep::Known(known) = step
                    && let KnownStep::Form { questions, .. } = known.as_ref()
                {
                    walk(questions, out);
                }
            }
        }

        let mut identifiers = Vec::new();
        walk(self.steps(), &mut identifiers);
        identifiers
    }
}

#[cfg(test)]
mod tests;
