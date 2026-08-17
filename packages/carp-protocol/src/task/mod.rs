// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tasks: what a protocol actually does once a trigger fires.
//!
//! Tasks fall into two families, and the difference is whether the participant
//! sees anything:
//!
//! - **background** tasks ([`KnownTask::Background`], [`KnownTask::Monitoring`])
//!   just collect their measures. Nothing appears in the app.
//! - **app** tasks ([`KnownTask::App`] and friends) put a card in the study
//!   app's task list, with a title, a description and an estimated duration,
//!   and collect their measures while the participant works through it.
//!
//! A task is identified by its `name`, which [`crate::control::TaskControl`]
//! refers to, so names must be unique within a protocol.

pub mod access;
pub mod kind;
pub mod measure;

use serde::{Deserialize, Serialize};

pub use kind::TaskKind;
pub use measure::{KnownMeasure, Measure};

use crate::duration::Micros;
use crate::node::UnknownNode;
use crate::survey::RpTask;

/// A task in a protocol.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Task {
    Known(Box<KnownTask>),
    /// A task class this version does not model. See [`crate::node`].
    Unknown(UnknownNode),
}

/// Fields every task carries: what it is called and what it collects.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskCore {
    pub name: String,
    #[serde(default)]
    pub measures: Vec<Measure>,
}

/// Fields shared by the tasks the participant interacts with.
///
/// Held as a struct rather than flattened so the variants that need it can
/// place it where CARP writes it, which differs between classes.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppTaskCore {
    /// Task category the app uses to pick an icon and a card style, e.g.
    /// `"survey"`, `"audio"`, `"health"`, `"cognitive"`.
    pub r#type: String,
    /// Heading on the task card. Often a localisation key.
    pub title: String,
    pub description: String,
    /// Longer text shown once the task is opened.
    #[serde(default)]
    pub instructions: String,
}

/// The task classes this crate models.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "__type")]
pub enum KnownTask {
    /// Collect measures without showing the participant anything.
    #[serde(rename = "dk.cachet.carp.common.application.tasks.BackgroundTask")]
    Background {
        #[serde(flatten)]
        core: TaskCore,
    },

    /// A background task carrying the protocol's own health measures - errors,
    /// triggered tasks, completed tasks. Every protocol wants exactly one.
    #[serde(rename = "dk.cachet.carp.common.application.tasks.MonitoringTask")]
    Monitoring {
        #[serde(flatten)]
        core: TaskCore,
    },

    /// A task the participant opens and completes.
    #[serde(
        rename = "dk.cachet.carp.common.application.tasks.AppTask",
        rename_all = "camelCase"
    )]
    App {
        #[serde(flatten)]
        core: TaskCore,
        #[serde(flatten)]
        app: AppTaskCore,
        /// Estimated minutes to complete, shown on the card.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        minutes_to_complete: Option<u32>,
        /// Whether to raise a phone notification when the task appears.
        #[serde(default)]
        notification: bool,
    },

    /// An app task presenting a Research Package survey. See [`crate::survey`].
    #[serde(
        rename = "dk.cachet.carp.common.application.tasks.RPAppTask",
        rename_all = "camelCase"
    )]
    RpApp {
        #[serde(flatten)]
        core: TaskCore,
        #[serde(flatten)]
        app: AppTaskCore,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        minutes_to_complete: Option<u32>,
        /// How long the task stays available before it disappears
        /// unanswered, in microseconds.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        expire: Option<Micros>,
        #[serde(default)]
        notification: bool,
        /// The survey itself.
        rp_task: RpTask,
    },

    /// An app task that reads metrics out of the phone's health database.
    #[serde(
        rename = "dk.cachet.carp.common.application.tasks.HealthAppTask",
        rename_all = "camelCase"
    )]
    HealthApp {
        #[serde(flatten)]
        core: TaskCore,
        #[serde(flatten)]
        app: AppTaskCore,
        #[serde(default)]
        notification: bool,
        /// Health metric names to read. Duplicates the `healthDataTypes` of
        /// the measure's sampling configuration, which is how CAMS reads it.
        #[serde(default)]
        types: Vec<String>,
    },

    /// A task that opens a web page, used by browser-delivered studies.
    #[serde(
        rename = "dk.cachet.carp.common.application.tasks.WebTask",
        rename_all = "camelCase"
    )]
    Web {
        #[serde(flatten)]
        core: TaskCore,
        /// Shown on the card; a `WebTask` has no separate title.
        description: String,
        /// Address the app opens when the participant starts the task.
        url: String,
    },
}

impl Task {
    /// The name the rest of the protocol refers to this task by.
    pub fn name(&self) -> &str {
        match self {
            Self::Known(task) => &task.core().name,
            Self::Unknown(node) => node.name().unwrap_or_default(),
        }
    }

    /// Rename the task. References are the caller's problem; use
    /// [`crate::builder`] to rename and re-point in one step.
    pub fn set_name(&mut self, name: impl Into<String>) {
        let name = name.into();
        match self {
            Self::Known(task) => task.core_mut().name = name,
            Self::Unknown(node) => {
                node.fields
                    .insert("name".to_owned(), serde_json::Value::String(name));
            }
        }
    }

    pub fn kind(&self) -> Option<TaskKind> {
        match self {
            Self::Known(task) => Some(task.kind()),
            Self::Unknown(_) => None,
        }
    }

    pub fn type_label(&self) -> &str {
        match self {
            Self::Known(task) => task.kind().label(),
            Self::Unknown(node) => node.short_type(),
        }
    }

    /// What this task collects.
    pub fn measures(&self) -> &[Measure] {
        match self {
            Self::Known(task) => &task.core().measures,
            Self::Unknown(_) => &[],
        }
    }

    pub fn measures_mut(&mut self) -> Option<&mut Vec<Measure>> {
        match self {
            Self::Known(task) => Some(&mut task.core_mut().measures),
            Self::Unknown(_) => None,
        }
    }

    /// The participant-facing fields, for the tasks that have them.
    pub fn app(&self) -> Option<&AppTaskCore> {
        match self {
            Self::Known(task) => task.app(),
            Self::Unknown(_) => None,
        }
    }

    pub fn app_mut(&mut self) -> Option<&mut AppTaskCore> {
        match self {
            Self::Known(task) => task.app_mut(),
            Self::Unknown(_) => None,
        }
    }

    /// The survey, for an `RPAppTask`.
    pub fn survey(&self) -> Option<&RpTask> {
        match self {
            Self::Known(task) => match task.as_ref() {
                KnownTask::RpApp { rp_task, .. } => Some(rp_task),
                _ => None,
            },
            Self::Unknown(_) => None,
        }
    }

    pub fn survey_mut(&mut self) -> Option<&mut RpTask> {
        match self {
            Self::Known(task) => match task.as_mut() {
                KnownTask::RpApp { rp_task, .. } => Some(rp_task),
                _ => None,
            },
            Self::Unknown(_) => None,
        }
    }

    /// Whether the participant sees this task in the app.
    pub fn is_visible_to_participant(&self) -> bool {
        self.kind().is_some_and(TaskKind::is_app_task)
    }
}

#[cfg(test)]
mod tests;
