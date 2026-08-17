// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Participant groups and their deployments, as returned by
//! `GET /api/studies/{study-id}/participantGroup/status`.
//!
//! The payload is kotlinx-serialised, so polymorphic values carry a `__type`
//! holding a fully qualified Kotlin class name such as
//! `dk.cachet.carp.deployments.application.StudyDeploymentStatus.Invited`.
//! Only the last segment is meaningful to a reader, so that is what the
//! accessors return.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::api::models::common::{CarpInstant, CarpUuid};

/// Last segment of a `__type` discriminator: `…StudyDeploymentStatus.Invited`
/// becomes `Invited`.
pub fn short_type(value: &str) -> &str {
    value.rsplit('.').next().unwrap_or(value).trim()
}

/// Participant groups of a study.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ParticipantGroupStatus {
    pub groups: Vec<ParticipantGroup>,
    /// Set when the endpoint answers with a bare status string rather than
    /// the group document.
    #[serde(skip)]
    pub label: Option<String>,
}

impl ParticipantGroupStatus {
    pub fn from_label(label: String) -> Self {
        Self {
            groups: Vec::new(),
            label: Some(label),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }

    /// How many groups sit in each deployment state, most common first.
    pub fn state_counts(&self) -> Vec<(String, usize)> {
        let mut counts: Vec<(String, usize)> = Vec::new();
        for group in &self.groups {
            let state = group.state().to_owned();
            match counts.iter_mut().find(|(name, _)| *name == state) {
                Some((_, count)) => *count += 1,
                None => counts.push((state, 1)),
            }
        }
        counts.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        counts
    }

    /// Which group each participant is deployed in.
    ///
    /// A deployment belongs to a participant group, and the group names its
    /// members in `participantStatusList`. That list is the only link between
    /// a participant and the deployment collecting their data, so it is what
    /// the index is built from.
    pub fn index_by_participant(&self) -> HashMap<String, usize> {
        let mut index = HashMap::new();
        for (position, group) in self.groups.iter().enumerate() {
            for participant in &group.deployment_status.participant_status_list {
                index.insert(participant.participant_id.to_string(), position);
            }
        }
        index
    }

    /// `3 groups · 2 invited · 1 running`, for the overview panel.
    pub fn summary(&self) -> String {
        if let Some(label) = &self.label {
            return label.clone();
        }
        if self.groups.is_empty() {
            return "no participant groups".to_owned();
        }
        let states = self
            .state_counts()
            .into_iter()
            .map(|(state, count)| format!("{count} {}", state.to_lowercase()))
            .collect::<Vec<_>>()
            .join(" · ");
        let plural = if self.groups.len() == 1 { "" } else { "s" };
        format!("{} group{plural} · {states}", self.groups.len())
    }
}

/// One invited group of participants and the deployment created for it.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ParticipantGroup {
    pub participant_group_id: CarpUuid,
    pub deployment_status: DeploymentStatus,
}

impl ParticipantGroup {
    /// `Invited`, `Running`, `Stopped`, …
    pub fn state(&self) -> &str {
        self.deployment_status.state()
    }

    pub fn short_id(&self) -> &str {
        self.participant_group_id.short()
    }

    pub fn participant_ids(&self) -> impl Iterator<Item = &str> {
        self.deployment_status
            .participant_status_list
            .iter()
            .map(|participant| participant.participant_id.as_str())
    }

    /// Primary devices this participant was asked to register, which is how
    /// their data reaches the deployment.
    pub fn assigned_devices(&self, participant_id: &str) -> &[String] {
        self.deployment_status
            .participant_status_list
            .iter()
            .find(|participant| participant.participant_id.as_str() == participant_id)
            .map_or(&[], |participant| {
                participant.assigned_primary_device_role_names.as_slice()
            })
    }

    /// `1/4` registered devices for this group's deployment.
    pub fn device_progress(&self) -> String {
        self.deployment_status.device_progress()
    }
}

/// Deployment lifecycle of one participant group.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DeploymentStatus {
    #[serde(rename = "__type")]
    pub kind: String,
    pub study_deployment_id: CarpUuid,
    pub created_on: Option<CarpInstant>,
    pub started_on: Option<CarpInstant>,
    pub device_status_list: Vec<DeviceStatus>,
    pub participant_status_list: Vec<ParticipantStatus>,
}

impl DeploymentStatus {
    pub fn state(&self) -> &str {
        short_type(&self.kind)
    }

    /// Devices that have been registered on a participant's phone.
    pub fn registered_devices(&self) -> usize {
        self.device_status_list
            .iter()
            .filter(|device| device.is_registered())
            .count()
    }

    /// `1/4` registered devices.
    pub fn device_progress(&self) -> String {
        format!(
            "{}/{}",
            self.registered_devices(),
            self.device_status_list.len()
        )
    }

    /// Devices still keeping this deployment from running.
    pub fn pending_devices(&self) -> Vec<&str> {
        self.device_status_list
            .iter()
            .filter(|device| !device.is_registered() && !device.device.is_optional)
            .map(|device| device.device.role_name.as_str())
            .collect()
    }
}

/// Registration state of a single device in a deployment.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DeviceStatus {
    #[serde(rename = "__type")]
    pub kind: String,
    pub device: DeviceInfo,
    pub can_be_deployed: bool,
    pub is_ready_for_deployment: bool,
}

impl DeviceStatus {
    /// `Unregistered`, `Registered`, `Deployed`, `NeedsRedeployment`.
    pub fn state(&self) -> &str {
        short_type(&self.kind)
    }

    pub fn is_registered(&self) -> bool {
        matches!(
            self.state(),
            "Registered" | "Deployed" | "NeedsRedeployment" | "Running"
        )
    }
}

/// A device taking part in a deployment.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DeviceInfo {
    #[serde(rename = "__type")]
    pub kind: String,
    pub role_name: String,
    pub is_primary_device: bool,
    pub is_optional: bool,
}

impl DeviceInfo {
    /// `Smartphone`, `LocationService`, `WeatherService`, …
    pub fn kind_label(&self) -> &str {
        short_type(&self.kind)
    }

    pub fn role(&self) -> &str {
        if self.role_name.is_empty() {
            self.kind_label()
        } else {
            &self.role_name
        }
    }
}

/// A participant taking part in a deployment.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ParticipantStatus {
    pub participant_id: CarpUuid,
    pub assigned_primary_device_role_names: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_status_is_summarised_not_dumped() {
        let status: ParticipantGroupStatus =
            serde_json::from_str(crate::fixtures::PARTICIPANT_GROUP_STATUS).unwrap();
        let group = &status.groups[0];

        assert_eq!(group.state(), "Invited");
        assert_eq!(group.short_id(), "df98d925");
        assert_eq!(group.deployment_status.device_progress(), "0/2");
        // The optional location service does not block the deployment.
        assert_eq!(group.deployment_status.pending_devices(), ["Primary Phone"]);
        assert_eq!(
            group.deployment_status.device_status_list[0].device.role(),
            "Primary Phone"
        );
        assert_eq!(
            group.deployment_status.device_status_list[1]
                .device
                .kind_label(),
            "LocationService"
        );
        assert_eq!(status.summary(), "1 group · 1 invited");
    }

    #[test]
    fn a_participant_resolves_to_their_deployment() {
        let status: ParticipantGroupStatus =
            serde_json::from_str(crate::fixtures::PARTICIPANT_GROUP_STATUS).unwrap();
        let index = status.index_by_participant();

        let position = index
            .get(crate::fixtures::PARTICIPANT_GROUP_MEMBER_ID)
            .copied()
            .expect("the participant is a member of the group");
        let group = &status.groups[position];

        assert_eq!(group.short_id(), "df98d925");
        assert_eq!(
            group.assigned_devices(crate::fixtures::PARTICIPANT_GROUP_MEMBER_ID),
            ["Primary Phone"]
        );
        // A participant of another study is not in this study's index.
        assert!(!index.contains_key("ffffffff-0000-0000-0000-000000000000"));
    }

    #[test]
    fn a_bare_status_string_is_kept_as_a_label() {
        let status = ParticipantGroupStatus::from_label("Running".to_owned());
        assert!(status.is_empty());
        assert_eq!(status.summary(), "Running");
    }
}
