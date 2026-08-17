// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! The Participants tab: who takes part, and what is asked of them.
//!
//! Roles above, expected data below. They are two lists rather than one
//! because the second is assigned to the first, and seeing both at once is
//! what makes an entry assigned to a deleted role obvious.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::Row;

use crate::studio::Studio;
use crate::ui::theme;
use crate::ui::widgets::{detail, master_detail, table};

/// Draw the roles, the expected data, and the panel describing the selection.
pub fn render(frame: &mut Frame, area: Rect, studio: &mut Studio) {
    let (list_area, detail_area) = master_detail(area, 3, 2);
    let [roles_area, expected_area] =
        Layout::vertical([Constraint::Fill(1), Constraint::Fill(2)]).areas(list_area);

    render_roles(frame, roles_area, studio);
    render_expected(frame, expected_area, studio);

    let Some(detail_area) = detail_area else {
        return;
    };
    let block = theme::block("role");
    match studio.lists.selected_role(&studio.protocol) {
        Some(role) => {
            frame.render_widget(
                detail::panel(block, role_lines(&role.role, studio)),
                detail_area,
            );
        }
        None => frame.render_widget(
            detail::empty(block, "no roles yet - press a to add one"),
            detail_area,
        ),
    }
}

fn render_roles(frame: &mut Frame, area: Rect, studio: &mut Studio) {
    let roles = &studio.protocol.participant_roles;
    let title = format!(
        "roles {}",
        table::position_label(&studio.lists.roles, roles.len())
    );

    if roles.is_empty() {
        frame.render_widget(
            table::placeholder("no roles yet - press a to add one", theme::block(&title)),
            area,
        );
        return;
    }

    let rows: Vec<Row> = roles
        .iter()
        .map(|role| {
            Row::new(vec![
                Line::styled(role.role.clone(), theme::value()),
                Line::styled(
                    if role.is_optional {
                        "optional"
                    } else {
                        "required"
                    }
                    .to_owned(),
                    theme::dim(),
                ),
            ])
        })
        .collect();

    let widths = vec![Constraint::Fill(3), Constraint::Length(10)];
    let list = table::table(
        table::header(["role", ""]),
        rows,
        widths,
        theme::focused_block(&title),
    );
    table::render(frame, area, list, &mut studio.lists.roles, roles.len());
}

fn render_expected(frame: &mut Frame, area: Rect, studio: &mut Studio) {
    let expected = &studio.protocol.expected_participant_data;
    let title = format!(
        "expected data {} · A add · E edit · X remove",
        table::position_label(&studio.lists.expected, expected.len())
    );

    if expected.is_empty() {
        frame.render_widget(
            table::placeholder(
                "nothing is asked of participants - press A to add an informed consent",
                theme::block(&title),
            ),
            area,
        );
        return;
    }

    let roles: Vec<&str> = studio
        .protocol
        .participant_roles
        .iter()
        .map(|role| role.role.as_str())
        .collect();

    let rows: Vec<Row> = expected
        .iter()
        .map(|entry| {
            // An entry assigned to a role that no longer exists is asked of
            // nobody, so it is coloured as the fault it is.
            let assigned = entry.assigned_to.label();
            let resolves = entry
                .assigned_to
                .role_names()
                .is_none_or(|names| names.iter().all(|name| roles.contains(&name.as_str())));

            Row::new(vec![
                Line::styled(
                    carp_protocol::node::short_type(entry.input_data_type()).to_owned(),
                    theme::value(),
                ),
                Line::styled(
                    assigned,
                    if resolves {
                        theme::dim()
                    } else {
                        theme::error()
                    },
                ),
            ])
        })
        .collect();

    let widths = vec![Constraint::Fill(3), Constraint::Fill(2)];
    let list = table::table(
        table::header(["asks for", "of"]),
        rows,
        widths,
        theme::block(&title),
    );
    table::render(
        frame,
        area,
        list,
        &mut studio.lists.expected,
        expected.len(),
    );
}

/// The selected role, and what is asked of it.
fn role_lines(role: &str, studio: &Studio) -> Vec<Line<'static>> {
    let protocol = &studio.protocol;
    let optional = protocol
        .participant_roles
        .iter()
        .find(|candidate| candidate.role == role)
        .is_some_and(|candidate| candidate.is_optional);

    let mut lines = vec![
        detail::field("role", role.to_owned()),
        detail::field("required", if optional { "no" } else { "yes" }),
        detail::blank(),
        detail::section("asked for"),
    ];

    let asked: Vec<&str> = protocol
        .expected_participant_data
        .iter()
        .filter(|entry| {
            entry
                .assigned_to
                .role_names()
                .is_none_or(|names| names.iter().any(|name| name == role))
        })
        .map(carp_protocol::participant::ExpectedParticipantData::input_data_type)
        .collect();

    if asked.is_empty() {
        lines.push(detail::note("  nothing"));
    } else {
        for input in &asked {
            lines.push(detail::bullet(
                carp_protocol::node::short_type(input).to_owned(),
                theme::value(),
            ));
        }
    }

    // The informed consent is the one entry whose absence has a consequence
    // outside the app, so it is called out rather than left to the checks.
    let has_consent = asked
        .iter()
        .any(|input| input.ends_with("informed_consent"));
    lines.push(detail::blank());
    lines.push(detail::field_styled(
        "informed consent",
        if has_consent {
            "expected"
        } else {
            "not expected"
        },
        if has_consent {
            theme::ok()
        } else {
            theme::warn()
        },
    ));
    if !has_consent {
        lines.push(detail::note(
            "  without it nothing is uploaded as a signed consent",
        ));
    }

    lines
}
