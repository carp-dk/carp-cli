// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! The Overview tab: what the protocol is, and where it stands.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::Line;

use crate::studio::Studio;
use crate::ui::theme;
use crate::ui::widgets::detail;

/// Draw the overview into `area`.
pub fn render(frame: &mut Frame, area: Rect, studio: &Studio) {
    let [left, right] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).areas(area);

    frame.render_widget(
        detail::panel(theme::block("protocol"), identity_lines(studio)),
        left,
    );
    frame.render_widget(
        detail::panel(theme::block("study app"), application_lines(studio)),
        right,
    );
}

/// Name, id, revision and size.
fn identity_lines(studio: &Studio) -> Vec<Line<'static>> {
    let protocol = &studio.protocol;
    let mut lines = vec![
        detail::field("name", protocol.name.clone()),
        detail::field(
            "description",
            protocol.description.clone().unwrap_or_else(|| "—".to_owned()),
        ),
        detail::blank(),
        detail::section("identity"),
        detail::field("id", protocol.id.clone()),
        detail::field("owner", protocol.owner_id.clone()),
        detail::field("created", protocol.created_on.clone()),
        detail::blank(),
        detail::section("version"),
        // The revision and the tag are different things and are shown as
        // such: the revision is CARP's counter, the tag is what CAWS files
        // the upload under.
        detail::field("revision", protocol.version.to_string()),
        detail::field_highlighted("next tag", studio.version_tag.to_string()),
        detail::blank(),
        detail::section("contents"),
        detail::field("summary", protocol.summary()),
        detail::field(
            "participants",
            format!(
                "{} role{}",
                protocol.participant_roles.len(),
                if protocol.participant_roles.len() == 1 { "" } else { "s" }
            ),
        ),
        detail::blank(),
        detail::field("file", studio.location()),
    ];

    if !studio.history.is_empty() {
        lines.push(detail::note(format!(
            "{} change{} can be undone",
            studio.history.depth(),
            if studio.history.depth() == 1 { "" } else { "s" }
        )));
    }
    lines
}

/// The CAMS `applicationData` block, or an invitation to create it.
fn application_lines(studio: &Studio) -> Vec<Line<'static>> {
    let Some(data) = studio.protocol.application_data.as_ref() else {
        return vec![
            detail::note("This protocol has no study-app settings."),
            detail::blank(),
            detail::text(
                "It targets the CARP runtime directly, as the browser-based \
                 studies do. Press A to add them.",
            ),
        ];
    };

    let mut lines = vec![
        detail::field(
            "api level",
            data.protocol_api_level.clone().unwrap_or_else(|| "unset".to_owned()),
        ),
        detail::field(
            "application",
            data.application_name.clone().unwrap_or_else(|| "—".to_owned()),
        ),
        detail::blank(),
    ];

    match data.study_description.as_ref() {
        Some(description) => {
            lines.push(detail::section("description"));
            lines.push(detail::field("title", description.title.clone()));
            lines.push(detail::field("purpose", description.purpose.clone()));
            if let Some(responsible) = description.responsible.as_ref() {
                lines.push(detail::blank());
                lines.push(detail::section("responsible"));
                lines.push(detail::field("name", responsible.name.clone()));
                lines.push(detail::field("email", responsible.email.clone()));
                lines.push(detail::field("affiliation", responsible.affiliation.clone()));
            }
        }
        None => lines.push(detail::note("no study description")),
    }

    lines.push(detail::blank());
    lines.push(detail::section("data endpoint"));
    match data.data_end_point.as_ref() {
        Some(endpoint) => lines.push(detail::field_highlighted("uploads to", endpoint.label())),
        None => lines.push(detail::note("the study app's default")),
    }

    lines.push(detail::blank());
    lines.push(detail::note("e edits the protocol · A edits these settings"));
    lines
}
