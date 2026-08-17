// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! One participant, opened from the participants tab: everything the API
//! reports, with room for the explanations the side panel cannot fit.

use ratatui::Frame;
use ratatui::layout::Rect;

use crate::app::state::ParticipantState;
use crate::ui::screens::study::deployments;
use crate::ui::widgets::{detail, master_detail};
use crate::ui::{icons, theme};
use carp_client::api::models::format_instant;

pub fn render(frame: &mut Frame, area: Rect, state: &ParticipantState) {
    let (left, right) = master_detail(area, 3, 2);
    render_participant(frame, left, state);
    if let Some(right) = right {
        render_notes(frame, right, state);
    }
}

fn render_participant(frame: &mut Frame, area: Rect, state: &ParticipantState) {
    let participant = &state.participant;
    let lines = vec![
        detail::section("identity"),
        detail::field("name", participant.display_name()),
        detail::field("identity", participant.identity().to_owned()),
        detail::field("participant id", participant.participant_id.clone()),
        detail::blank(),
        detail::section("status"),
        detail::field_styled(
            "deployment",
            icons::with(
                icons::device(participant.deployed, false),
                participant.deployment_label(),
            ),
            if participant.deployed {
                theme::ok()
            } else {
                theme::warn()
            },
        ),
        detail::field_highlighted("account", participant.account_label().to_owned()),
        detail::field("invited", format_instant(participant.invited_on)),
        detail::blank(),
        detail::section("study"),
        detail::field("name", state.study.name.clone()),
        detail::field("study id", state.study.study_id.to_string()),
        detail::field("stage", state.study.stage().to_owned()),
    ];

    frame.render_widget(
        detail::panel(theme::focused_block("Participant"), lines),
        area,
    );
}

fn render_notes(frame: &mut Frame, area: Rect, state: &ParticipantState) {
    let participant = &state.participant;
    let mut lines = Vec::new();

    match &state.group {
        Some(group) => {
            let status = &group.deployment_status;
            lines.push(detail::field(
                "group",
                group.participant_group_id.to_string(),
            ));
            lines.push(detail::field_styled(
                "state",
                icons::with(icons::deployment_state(group.state()), group.state()),
                deployments::state_style(group.state()),
            ));
            lines.push(detail::field("created", format_instant(status.created_on)));
            lines.push(detail::field("started", format_instant(status.started_on)));
            lines.push(detail::blank());

            let assigned = group.assigned_devices(&participant.participant_id);
            lines.push(detail::section(&format!(
                "devices ({} registered)",
                status.device_progress()
            )));
            for device in &status.device_status_list {
                // Mark the devices this participant is responsible for.
                let mine = assigned.iter().any(|role| role == &device.device.role_name);
                lines.push(detail::bullet(
                    icons::with(
                        icons::device(device.is_registered(), device.device.is_optional),
                        format!(
                            "{}{} · {}",
                            device.device.role(),
                            if mine { " (theirs)" } else { "" },
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
        }
        None => lines.push(detail::note(
            "No participant group lists this participant, so no deployment is \
             collecting their data yet.",
        )),
    }

    lines.push(detail::blank());
    let lines = [
        lines,
        vec![
        detail::section("data"),
        detail::note(
            "Collected data is downloaded per study rather than per participant: open the Exports \
             tab, press n to request an export, and download the archive once it is available.",
        ),
        detail::blank(),
        detail::note("esc returns to the participant list"),
        ],
    ]
    .concat();

    frame.render_widget(detail::panel(theme::block("Deployment"), lines), area);
}
