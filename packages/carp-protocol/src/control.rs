// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! The wiring between triggers, tasks and devices.
//!
//! CARP keeps triggers and tasks in separate lists and joins them with a third:
//! a [`TaskControl`] says *this trigger* starts *that task* on *this device*.
//! The indirection is what lets one trigger start several tasks, and one task
//! be started by several triggers.
//!
//! Nothing in the JSON checks that the three names resolve. A control naming a
//! deleted task is perfectly valid JSON and a broken protocol, which is why
//! [`mod@crate::validate`] checks them and [`crate::builder`] maintains them.

use serde::{Deserialize, Serialize};

/// Which primary device a connected device is reached through.
///
/// `role_name` is the connected device; `connected_to_role_name` is the
/// primary one it hangs off.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceConnection {
    /// Role name of the connected device.
    pub role_name: String,
    /// Role name of the primary device it connects through.
    pub connected_to_role_name: String,
}

impl DeviceConnection {
    pub fn new(role_name: impl Into<String>, connected_to: impl Into<String>) -> Self {
        Self {
            role_name: role_name.into(),
            connected_to_role_name: connected_to.into(),
        }
    }
}

/// What a trigger does to a task.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Control {
    /// Begin the task. Every control in the reference protocols is a `Start`.
    #[default]
    Start,
    /// End a task that is already running.
    Stop,
}

impl Control {
    pub const ALL: [Self; 2] = [Self::Start, Self::Stop];

    pub fn label(self) -> &'static str {
        match self {
            Self::Start => "Start",
            Self::Stop => "Stop",
        }
    }
}

/// One trigger starting (or stopping) one task on one device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskControl {
    /// Key of the trigger in [`crate::StudyProtocol::triggers`].
    pub trigger_id: u32,
    /// Name of the task in [`crate::StudyProtocol::tasks`].
    pub task_name: String,
    /// Role name of the device the task runs on. Not necessarily the device
    /// the trigger is evaluated on: a phone can be told to start collecting
    /// from a chest strap.
    pub destination_device_role_name: String,
    #[serde(default)]
    pub control: Control,
}

impl TaskControl {
    /// A control starting `task_name` on `device` when trigger `trigger_id`
    /// fires.
    pub fn start(trigger_id: u32, task_name: impl Into<String>, device: impl Into<String>) -> Self {
        Self {
            trigger_id,
            task_name: task_name.into(),
            destination_device_role_name: device.into(),
            control: Control::Start,
        }
    }
}

#[cfg(test)]
mod tests;
