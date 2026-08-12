// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! [`StudyProtocol`]: the root of a `protocol.json` document.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::application_data::ApplicationData;
use crate::control::{DeviceConnection, TaskControl};
use crate::device::Device;
use crate::participant::{ExpectedParticipantData, ParticipantRole};
use crate::task::Task;
use crate::trigger::Trigger;

/// A complete study protocol, matching CARP's `StudyProtocolSnapshot`.
///
/// Field order follows the documents the Dart generator produced, so a
/// protocol migrated into this tool and written back gives a small diff.
///
/// # Shape of the graph
///
/// The document is a set of nodes plus the edges between them, and the edges
/// are all by *name*:
///
/// - a [`Trigger`] names the device it fires on (`sourceDeviceRoleName`)
/// - a [`TaskControl`] names a trigger by id, a [`Task`] by name, and the
///   device it runs on by role name
/// - a [`DeviceConnection`] names two devices by role name
///
/// Nothing in the JSON enforces that those names resolve, which is why
/// [`mod@crate::validate`] exists and why [`crate::builder`] renames and deletes
/// through methods that fix up the references.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyProtocol {
    /// CARP Mobile Sensing extensions. Absent in protocols that target the
    /// core runtime only, such as the browser-based ICAT study.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub application_data: Option<ApplicationData>,

    /// Protocol identity. Stable across versions: a new version of the same
    /// protocol keeps the id and increments [`StudyProtocol::version`].
    pub id: String,
    /// When the first version was authored, as an ISO-8601 instant.
    pub created_on: String,
    /// Revision counter, starting at 0. See [`crate::version`].
    pub version: u32,

    /// Free-text description. Several protocols put a localisation key here
    /// rather than prose.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// UUID of the account that owns the protocol. CAWS replaces it with the
    /// uploading user's id, so any valid UUID works while authoring.
    pub owner_id: String,
    pub name: String,

    #[serde(default)]
    pub participant_roles: Vec<ParticipantRole>,
    /// Devices that run the study themselves - a phone, or a browser.
    #[serde(default)]
    pub primary_devices: Vec<Device>,
    /// Devices and services reached through a primary device.
    #[serde(default)]
    pub connected_devices: Vec<Device>,
    /// Which connected device hangs off which primary device.
    #[serde(default)]
    pub connections: Vec<DeviceConnection>,
    /// Participant role -> device role names assigned to it. Empty in every
    /// reference protocol; CAWS assigns devices at deployment time.
    #[serde(default)]
    pub assigned_devices: BTreeMap<String, BTreeSet<String>>,

    #[serde(default)]
    pub tasks: Vec<Task>,
    /// Triggers by id. `serde_json` writes integer keys as the JSON strings
    /// CARP expects (`"0"`, `"1"`), and a `BTreeMap` keeps them in numeric
    /// order rather than the lexicographic order string keys would give.
    #[serde(default)]
    pub triggers: BTreeMap<u32, Trigger>,
    /// Which trigger starts which task on which device.
    #[serde(default)]
    pub task_controls: Vec<TaskControl>,
    #[serde(default)]
    pub expected_participant_data: Vec<ExpectedParticipantData>,
}

impl StudyProtocol {
    /// An empty protocol with a fresh id, owned by `owner_id`.
    ///
    /// `created_on` is the current time; the caller can overwrite it when
    /// reproducing an existing document.
    pub fn new(name: impl Into<String>, owner_id: impl Into<String>) -> Self {
        Self {
            application_data: Some(ApplicationData::default()),
            id: uuid::Uuid::new_v4().to_string(),
            created_on: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
            version: 0,
            description: None,
            owner_id: owner_id.into(),
            name: name.into(),
            participant_roles: Vec::new(),
            primary_devices: Vec::new(),
            connected_devices: Vec::new(),
            connections: Vec::new(),
            assigned_devices: BTreeMap::new(),
            tasks: Vec::new(),
            triggers: BTreeMap::new(),
            task_controls: Vec::new(),
            expected_participant_data: Vec::new(),
        }
    }

    /// Every device, primary first, as the editor lists them.
    pub fn devices(&self) -> impl Iterator<Item = &Device> {
        self.primary_devices.iter().chain(&self.connected_devices)
    }

    /// Role names of every device, in the same order.
    pub fn device_role_names(&self) -> Vec<String> {
        self.devices()
            .map(|device| device.role_name().to_owned())
            .collect()
    }

    /// The device with this role name, primary or connected.
    pub fn device(&self, role_name: &str) -> Option<&Device> {
        self.devices()
            .find(|device| device.role_name() == role_name)
    }

    /// Names of every task, in document order.
    pub fn task_names(&self) -> Vec<String> {
        self.tasks
            .iter()
            .map(|task| task.name().to_owned())
            .collect()
    }

    pub fn task(&self, name: &str) -> Option<&Task> {
        self.tasks.iter().find(|task| task.name() == name)
    }

    /// The lowest trigger id not already in use.
    ///
    /// Ids are dense in the reference protocols but nothing requires it, so
    /// this fills a gap left by a deletion rather than always appending.
    pub fn next_trigger_id(&self) -> u32 {
        (0u32..)
            .find(|id| !self.triggers.contains_key(id))
            .unwrap_or(0)
    }

    /// The task controls that reference `trigger_id`.
    pub fn controls_for_trigger(&self, trigger_id: u32) -> impl Iterator<Item = &TaskControl> {
        self.task_controls
            .iter()
            .filter(move |control| control.trigger_id == trigger_id)
    }

    /// The task controls that reference the task named `task_name`.
    pub fn controls_for_task<'a>(
        &'a self,
        task_name: &'a str,
    ) -> impl Iterator<Item = &'a TaskControl> {
        self.task_controls
            .iter()
            .filter(move |control| control.task_name == task_name)
    }

    /// A short line describing the protocol's size, for list views.
    pub fn summary(&self) -> String {
        format!(
            "{} device{}, {} task{}, {} trigger{}",
            self.devices().count(),
            plural(self.devices().count()),
            self.tasks.len(),
            plural(self.tasks.len()),
            self.triggers.len(),
            plural(self.triggers.len()),
        )
    }
}

fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}

#[cfg(test)]
mod tests;
