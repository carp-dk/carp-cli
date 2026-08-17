// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Triggers: when a task starts.
//!
//! Every trigger names the device it is evaluated on
//! (`sourceDeviceRoleName`), and the protocol stores them in a map keyed by
//! id. A [`crate::control::TaskControl`] then joins a trigger id to a task
//! name, which is why adding a task involves adding a trigger too - the two
//! are separate rows that a control ties together.
//!
//! The kinds divide by what they wait for:
//!
//! - **nothing**: [`KnownTrigger::Immediate`] fires as soon as the study
//!   starts and never again; [`KnownTrigger::OneTime`] the same but only on
//!   the very first run; [`KnownTrigger::NoOp`] never fires, and marks a task
//!   the app starts itself
//! - **the clock**: [`KnownTrigger::Periodic`],
//!   [`KnownTrigger::RecurrentScheduled`], [`KnownTrigger::CronScheduled`]
//! - **the participant**: [`KnownTrigger::UserTask`] and
//!   [`KnownTrigger::NoUserTask`] watch another task's state
//! - **the data**: [`KnownTrigger::SamplingEvent`] fires when a measure
//!   matches a condition

pub mod access;
pub mod kind;
pub mod schedule;

use serde::{Deserialize, Serialize};

pub use kind::TriggerKind;
pub use schedule::{DayOfWeek, Recurrence, TimeOfDay};

use crate::duration::Micros;
use crate::node::UnknownNode;

/// A trigger in a protocol.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Trigger {
    Known(Box<KnownTrigger>),
    /// A trigger class this version does not model. See [`crate::node`].
    Unknown(UnknownNode),
}

/// The trigger classes this crate models.
///
/// Field order within each variant follows what CARP writes, so a protocol
/// read and written back is unchanged.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// `rename_all_fields` renames the fields of every variant; a bare
// `rename_all` would rename the variants, which are already renamed one by
// one to their fully qualified Kotlin class.
#[serde(tag = "__type", rename_all_fields = "camelCase")]
pub enum KnownTrigger {
    /// Never fires. Used to park a task that the app starts by itself, and to
    /// hold the protocol's monitoring task.
    #[serde(rename = "dk.cachet.carp.common.application.triggers.NoOpTrigger")]
    NoOp { source_device_role_name: String },

    /// Fires once when the study starts, and again on every app restart.
    #[serde(rename = "dk.cachet.carp.common.application.triggers.ImmediateTrigger")]
    Immediate { source_device_role_name: String },

    /// Fires once, on the first run of the study only.
    #[serde(rename = "dk.cachet.carp.common.application.triggers.OneTimeTrigger")]
    OneTime { source_device_role_name: String },

    /// Fires every `period`, measured from when the study started.
    #[serde(rename = "dk.cachet.carp.common.application.triggers.PeriodicTrigger")]
    Periodic {
        source_device_role_name: String,
        period: Micros,
    },

    /// Fires at a wall-clock time, daily to monthly.
    #[serde(rename = "dk.cachet.carp.common.application.triggers.RecurrentScheduledTrigger")]
    RecurrentScheduled {
        source_device_role_name: String,
        /// `"daily"`, `"weekly"`, `"biweekly"` or `"monthly"`. See
        /// [`Recurrence`].
        r#type: String,
        time: TimeOfDay,
        /// Periods to skip between firings; 0 means every period.
        #[serde(default)]
        separation_count: u32,
        /// 1 (Monday) to 7 (Sunday). Only written for weekly recurrences.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        day_of_week: Option<u8>,
        /// The recurrence expressed as a duration. Must agree with `type`.
        period: Micros,
    },

    /// Fires on a cron expression, for schedules the recurrence types cannot
    /// express.
    #[serde(rename = "dk.cachet.carp.common.application.triggers.CronScheduledTrigger")]
    CronScheduled {
        source_device_role_name: String,
        /// Standard five-field cron, e.g. `0 10 * * *` for 10:00 daily.
        cron_expression: String,
    },

    /// Fires when another task reaches a state, typically `"done"`.
    #[serde(rename = "dk.cachet.carp.common.application.triggers.UserTaskTrigger")]
    UserTask {
        source_device_role_name: String,
        /// The task being watched.
        task_name: String,
        /// `"done"`, `"started"` or another `UserTaskState`.
        trigger_condition: String,
    },

    /// Fires when a task is *removed* from the participant's task list,
    /// which is how a protocol chains "when they finish this, offer that".
    #[serde(rename = "dk.cachet.carp.common.application.triggers.NoUserTaskTrigger")]
    NoUserTask {
        source_device_role_name: String,
        task_name: String,
    },

    /// Fires when a measure produces data matching a condition.
    #[serde(rename = "dk.cachet.carp.common.application.triggers.SamplingEventTrigger")]
    SamplingEvent {
        source_device_role_name: String,
        /// The measure watched, e.g. `dk.cachet.carp.movesense.state`.
        measure_type: String,
        /// The data value that has to match. Its shape depends entirely on
        /// `measure_type`, so it is carried as raw JSON rather than modelled.
        trigger_condition: serde_json::Value,
    },
}

impl Trigger {
    /// A trigger that fires as soon as the study starts, on `device`.
    pub fn immediate(device: impl Into<String>) -> Self {
        Self::Known(Box::new(KnownTrigger::Immediate {
            source_device_role_name: device.into(),
        }))
    }

    /// A trigger that never fires, on `device`.
    pub fn no_op(device: impl Into<String>) -> Self {
        Self::Known(Box::new(KnownTrigger::NoOp {
            source_device_role_name: device.into(),
        }))
    }

    /// The device this trigger is evaluated on.
    pub fn source_device(&self) -> &str {
        match self {
            Self::Known(trigger) => trigger.source_device(),
            Self::Unknown(node) => node
                .field("sourceDeviceRoleName")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default(),
        }
    }

    /// Point the trigger at a different device.
    pub fn set_source_device(&mut self, device: impl Into<String>) {
        let device = device.into();
        match self {
            Self::Known(trigger) => trigger.set_source_device(device),
            Self::Unknown(node) => {
                node.fields.insert(
                    "sourceDeviceRoleName".to_owned(),
                    serde_json::Value::String(device),
                );
            }
        }
    }

    /// The task this trigger watches, for the kinds that watch one.
    ///
    /// Renaming a task has to update these as well as the task controls,
    /// which is why [`crate::builder`] owns renaming.
    pub fn watched_task(&self) -> Option<&str> {
        match self {
            Self::Known(trigger) => match trigger.as_ref() {
                KnownTrigger::UserTask { task_name, .. }
                | KnownTrigger::NoUserTask { task_name, .. } => Some(task_name),
                _ => None,
            },
            Self::Unknown(node) => node.field("taskName").and_then(serde_json::Value::as_str),
        }
    }

    /// Rename the watched task, if this trigger watches one.
    pub fn set_watched_task(&mut self, name: &str) {
        match self {
            Self::Known(trigger) => match trigger.as_mut() {
                KnownTrigger::UserTask { task_name, .. }
                | KnownTrigger::NoUserTask { task_name, .. } => name.clone_into(task_name),
                _ => {}
            },
            Self::Unknown(node) => {
                if node.fields.contains_key("taskName") {
                    node.fields.insert(
                        "taskName".to_owned(),
                        serde_json::Value::String(name.to_owned()),
                    );
                }
            }
        }
    }

    pub fn kind(&self) -> Option<TriggerKind> {
        match self {
            Self::Known(trigger) => Some(trigger.kind()),
            Self::Unknown(_) => None,
        }
    }

    pub fn type_label(&self) -> &str {
        match self {
            Self::Known(trigger) => trigger.kind().label(),
            Self::Unknown(node) => node.short_type(),
        }
    }

    /// A phrase describing when this trigger fires, for the editor's list.
    pub fn schedule_label(&self) -> String {
        let Self::Known(trigger) = self else {
            return "unknown schedule".to_owned();
        };
        match trigger.as_ref() {
            KnownTrigger::NoOp { .. } => "never (started by the app)".to_owned(),
            KnownTrigger::Immediate { .. } => "when the study starts".to_owned(),
            KnownTrigger::OneTime { .. } => "once, on first run".to_owned(),
            KnownTrigger::Periodic { period, .. } => format!("every {}", period.human()),
            KnownTrigger::RecurrentScheduled {
                r#type,
                time,
                day_of_week,
                ..
            } => match day_of_week.map(DayOfWeek) {
                Some(day) => format!("{type} on {} at {}", day.label(), time.label()),
                None => format!("{type} at {}", time.label()),
            },
            KnownTrigger::CronScheduled {
                cron_expression, ..
            } => format!("cron {cron_expression}"),
            KnownTrigger::UserTask {
                task_name,
                trigger_condition,
                ..
            } => format!("when \"{task_name}\" is {trigger_condition}"),
            KnownTrigger::NoUserTask { task_name, .. } => {
                format!("when \"{task_name}\" leaves the list")
            }
            KnownTrigger::SamplingEvent { measure_type, .. } => {
                format!("on {}", crate::node::short_type(measure_type))
            }
        }
    }
}

#[cfg(test)]
mod tests;
