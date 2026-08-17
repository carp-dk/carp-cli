// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! `carp deployments` - a study's participant groups and their devices.
//!
//! CARP has no endpoint that lists deployments as such: a deployment is created
//! for a participant group, and `participantGroup/status` is where they are
//! reported. Both commands here read that one document.

use carp_client::ApiError;
use carp_client::api::endpoints::studies;
use carp_client::api::models::{ParticipantGroup, format_instant};
use color_eyre::Result;
use serde::Serialize;

use crate::cli::{DeploymentsCommand, Global};
use crate::commands::{Session, connect};
use crate::output::{self, Rows};

pub async fn run(command: &DeploymentsCommand, global: &Global) -> Result<()> {
    let session = connect(global).await?;
    match command {
        DeploymentsCommand::List { study } => list(&session, study).await,
        DeploymentsCommand::Show { study, deployment } => show(&session, study, deployment).await,
    }
}

/// A deployment as a row: the group's own id is of little use on its own, so
/// the deployment id leads.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DeploymentRow {
    deployment_id: String,
    participant_group_id: String,
    state: String,
    devices: String,
    participants: usize,
    created: Option<String>,
    started: Option<String>,
    pending_devices: Vec<String>,
}

impl From<&ParticipantGroup> for DeploymentRow {
    fn from(group: &ParticipantGroup) -> Self {
        let status = &group.deployment_status;
        Self {
            deployment_id: status.study_deployment_id.to_string(),
            participant_group_id: group.participant_group_id.to_string(),
            state: group.state().to_owned(),
            devices: group.device_progress(),
            participants: status.participant_status_list.len(),
            created: status.created_on.map(|when| when.to_local_string()),
            started: status.started_on.map(|when| when.to_local_string()),
            pending_devices: status
                .pending_devices()
                .into_iter()
                .map(ToOwned::to_owned)
                .collect(),
        }
    }
}

impl Rows for DeploymentRow {
    const HEADERS: &'static [&'static str] = &[
        "deployment",
        "state",
        "devices",
        "participants",
        "created",
        "waiting on",
    ];

    fn cells(&self) -> Vec<String> {
        vec![
            self.deployment_id.clone(),
            self.state.clone(),
            self.devices.clone(),
            self.participants.to_string(),
            self.created.clone().unwrap_or_else(|| "-".to_owned()),
            if self.pending_devices.is_empty() {
                "-".to_owned()
            } else {
                self.pending_devices.join(", ")
            },
        ]
    }
}

async fn groups(session: &Session, study: &str) -> Result<Vec<ParticipantGroup>> {
    Ok(studies::participant_group_status(&session.client, study)
        .await?
        .groups)
}

async fn list(session: &Session, study: &str) -> Result<()> {
    let rows: Vec<DeploymentRow> = groups(session, study)
        .await?
        .iter()
        .map(Into::into)
        .collect();
    output::rows(&rows, session.format)
}

/// One deployment in full, with every device and participant on it.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DeploymentDetail {
    #[serde(flatten)]
    row: DeploymentRow,
    devices: Vec<DeviceRow>,
    participants: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DeviceRow {
    role_name: String,
    kind: String,
    state: String,
    is_primary: bool,
    is_optional: bool,
}

async fn show(session: &Session, study: &str, deployment: &str) -> Result<()> {
    let group = groups(session, study)
        .await?
        .into_iter()
        // Addressable by either id: the deployment id is what the data
        // endpoints take, the group id is what the portal shows.
        .find(|group| {
            group.deployment_status.study_deployment_id.as_str() == deployment
                || group.participant_group_id.as_str() == deployment
        })
        .ok_or_else(|| {
            ApiError::NotFound(format!("no deployment {deployment} in study {study}"))
        })?;

    let status = &group.deployment_status;
    let detail = DeploymentDetail {
        devices: status
            .device_status_list
            .iter()
            .map(|device| DeviceRow {
                role_name: device.device.role().to_owned(),
                kind: device.device.kind_label().to_owned(),
                state: device.state().to_owned(),
                is_primary: device.device.is_primary_device,
                is_optional: device.device.is_optional,
            })
            .collect(),
        participants: group.participant_ids().map(ToOwned::to_owned).collect(),
        row: DeploymentRow::from(&group),
    };

    let lines = vec![
        ("deployment", detail.row.deployment_id.clone()),
        ("group", detail.row.participant_group_id.clone()),
        ("state", detail.row.state.clone()),
        ("created", format_instant(status.created_on)),
        (
            "started",
            status
                .started_on
                .map_or_else(|| "-".to_owned(), |when| when.to_local_string()),
        ),
        ("devices", detail.row.devices.clone()),
        (
            "registered",
            detail
                .devices
                .iter()
                .map(|device| format!("{} ({})", device.role_name, device.state))
                .collect::<Vec<_>>()
                .join(", "),
        ),
        (
            "waiting on",
            if detail.row.pending_devices.is_empty() {
                "-".to_owned()
            } else {
                detail.row.pending_devices.join(", ")
            },
        ),
        (
            "participants",
            if detail.participants.is_empty() {
                "-".to_owned()
            } else {
                detail.participants.join(", ")
            },
        ),
    ];
    output::detail(&detail, &lines, session.format)
}
