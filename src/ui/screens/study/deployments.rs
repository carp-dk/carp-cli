// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Participant groups and the state of the deployment created for each one.
//!
//! This is the readable form of `participantGroup/status`, which the API
//! returns as a deeply nested kotlinx document.

use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::Row;

use crate::app::state::StudyState;
use crate::ui::screens::study::{DETAIL_WEIGHT, LIST_WEIGHT, tab_title};
use crate::ui::widgets::{detail, master_detail, table};
use crate::ui::{icons, theme};
use carp_client::api::models::{ParticipantGroup, ParticipantSummary, format_instant};

pub fn render(frame: &mut Frame, area: Rect, study: &mut StudyState, ticks: usize) {
    let (list_area, detail_area) = master_detail(area, LIST_WEIGHT, DETAIL_WEIGHT);
    if let Some(detail_area) = detail_area {
        render_detail(frame, detail_area, study);
    }
    render_list(frame, list_area, study, ticks);
}

fn render_list(frame: &mut Frame, area: Rect, study: &mut StudyState, ticks: usize) {
    let title = tab_title(
        "Deployments",
        table::position_label(&study.groups_table, study.groups().groups.len()),
        ticks,
        study.details_loading,
    );
    let block = theme::focused_block(&title);

    if study.groups().groups.is_empty() {
        let message = if study.details_loading {
            "loading deployments…"
        } else if let Some(label) = &study.groups().label {
            // The endpoint answered with a plain status rather than groups.
            label.as_str()
        } else {
            "no participant groups have been invited for this study"
        };
        frame.render_widget(table::placeholder(message, block), area);
        return;
    }

    let rows: Vec<Row> = study
        .groups()
        .groups
        .iter()
        .map(|group| {
            let status = &group.deployment_status;
            Row::new(vec![
                // Whose deployment this is, which is what a reader is after.
                Line::raw(members_label(study, group)),
                Line::styled(
                    icons::with(icons::deployment_state(group.state()), group.state()),
                    state_style(group.state()),
                ),
                Line::raw(status.device_progress()),
                Line::raw(
                    status
                        .created_on
                        .map(|created| created.to_local_date())
                        .unwrap_or_else(|| "-".to_owned()),
                ),
            ])
        })
        .collect();

    let header = table::header(["Participants", "State", "Devices", "Created"]);
    let widths = vec![
        Constraint::Fill(1),
        Constraint::Length(12 + icons::cell_width()),
        Constraint::Length(8),
        Constraint::Length(10),
    ];

    let len = rows.len();
    table::render(
        frame,
        area,
        table::table(header, rows, widths, block),
        &mut study.groups_table,
        len,
    );
}

fn render_detail(frame: &mut Frame, area: Rect, study: &StudyState) {
    let block = theme::block("Deployment");

    let Some(group) = study.selected_group() else {
        frame.render_widget(detail::empty(block, "no deployment selected"), area);
        return;
    };

    let status = &group.deployment_status;
    let mut lines = vec![
        detail::section("group"),
        detail::field("group id", group.participant_group_id.to_string()),
    ];
    // CARP reuses the group id as the deployment id; only show it when it
    // actually differs, rather than printing the same UUID twice.
    if status.study_deployment_id != group.participant_group_id {
        lines.push(detail::field(
            "deployment id",
            status.study_deployment_id.to_string(),
        ));
    }
    lines.extend([
        detail::field_styled(
            "state",
            icons::with(icons::deployment_state(group.state()), group.state()),
            state_style(group.state()),
        ),
        detail::field("created", format_instant(status.created_on)),
        detail::field("started", format_instant(status.started_on)),
        detail::blank(),
        detail::section(&format!(
            "devices ({} registered)",
            status.device_progress()
        )),
    ]);

    if status.device_status_list.is_empty() {
        lines.push(detail::note("no devices in this deployment"));
    }
    for device in &status.device_status_list {
        let optional = if device.device.is_optional {
            ", optional"
        } else {
            ""
        };
        lines.push(detail::bullet(
            icons::with(
                icons::device(device.is_registered(), device.device.is_optional),
                format!(
                    "{} · {} · {}{optional}",
                    device.device.role(),
                    device.device.kind_label(),
                    device.state().to_lowercase()
                ),
            ),
            if device.is_registered() {
                theme::ok()
            } else if device.device.is_optional {
                theme::dim()
            } else {
                theme::warn()
            },
        ));
    }

    let pending = status.pending_devices();
    if !pending.is_empty() {
        lines.push(detail::blank());
        lines.push(detail::section("waiting for"));
        lines.push(detail::text(pending.join(", ")));
    }

    lines.push(detail::blank());
    lines.push(detail::section(&format!(
        "participants ({})",
        status.participant_status_list.len()
    )));
    for participant in &status.participant_status_list {
        let id = participant.participant_id.as_str();
        // Resolve the id against the participants already loaded; fall back to
        // the id itself rather than showing nothing.
        let known = study.participants.lookup(id);
        let name = known.map_or_else(
            || participant.participant_id.short().to_owned(),
            ParticipantSummary::display_name,
        );
        lines.push(detail::bullet(
            icons::with(icons::participants(), name),
            theme::value(),
        ));
        if let Some(known) = known {
            lines.push(detail::note(format!("    {}", known.identity())));
        }
        if !participant.assigned_primary_device_role_names.is_empty() {
            lines.push(detail::note(format!(
                "    on {}",
                participant.assigned_primary_device_role_names.join(", ")
            )));
        }
    }
    if study.participants.directory.is_empty() && !status.participant_status_list.is_empty() {
        lines.push(detail::blank());
        lines.push(detail::note("loading participant names…"));
    }

    frame.render_widget(detail::panel(block, lines), area);
}

/// Who a deployment belongs to: one name, or the first name and a count.
fn members_label(study: &StudyState, group: &ParticipantGroup) -> String {
    let members = study.group_members(group);
    match members.split_first() {
        None => "no participants".to_owned(),
        Some((first, [])) => first.clone(),
        Some((first, rest)) => format!("{first} +{}", rest.len()),
    }
}

/// Deployment lifecycle colours, shared with the overview summary.
pub fn state_style(state: &str) -> Style {
    match state {
        "Running" | "DeviceDeployed" => theme::ok(),
        "Invited" | "DeployingDevices" => theme::warn(),
        "Stopped" => theme::dim(),
        _ => theme::value(),
    }
}
