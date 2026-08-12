// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Researchers and research assistants attached to the study.

use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::text::Line;
use ratatui::widgets::Row;

use crate::api::models::Account;
use crate::app::state::StudyState;
use crate::ui::screens::study::{DETAIL_WEIGHT, LIST_WEIGHT, tab_title};
use crate::ui::widgets::{detail, master_detail, table};
use crate::ui::{icons, theme};

pub fn render(frame: &mut Frame, area: Rect, study: &mut StudyState, ticks: usize) {
    let (list_area, detail_area) = master_detail(area, LIST_WEIGHT, DETAIL_WEIGHT);
    if let Some(detail_area) = detail_area {
        render_detail(frame, detail_area, study.selected_staff());
    }
    render_list(frame, list_area, study, ticks);
}

fn render_list(frame: &mut Frame, area: Rect, study: &mut StudyState, ticks: usize) {
    let members = study.staff();
    let title = tab_title(
        "Staff",
        table::position_label(&study.staff_table, members.len()),
        ticks,
        study.details_loading,
    );
    let block = theme::focused_block(&title);

    if members.is_empty() {
        let message = if study.details_loading {
            "loading staff…"
        } else {
            "no researchers are attached to this study"
        };
        frame.render_widget(table::placeholder(message, block), area);
        return;
    }

    let rows: Vec<Row> = members
        .iter()
        .map(|(account, role)| {
            Row::new(vec![
                Line::raw(icons::with(icons::staff(), account.display_name())),
                Line::raw(account.identifier().to_owned()),
                Line::styled(
                    (*role).to_owned(),
                    if *role == "researcher" {
                        theme::ok()
                    } else {
                        theme::value()
                    },
                ),
            ])
        })
        .collect();

    let header = table::header(["Name", "Email", "On this study"]);
    let widths = vec![
        Constraint::Fill(2),
        Constraint::Fill(3),
        Constraint::Length(19),
    ];

    let len = rows.len();
    table::render(
        frame,
        area,
        table::table(header, rows, widths, block),
        &mut study.staff_table,
        len,
    );
}

fn render_detail(frame: &mut Frame, area: Rect, member: Option<(&Account, &'static str)>) {
    let block = theme::block("Account");

    let Some((account, role)) = member else {
        frame.render_widget(detail::empty(block, "no account selected"), area);
        return;
    };

    let lines = vec![
        detail::section("account"),
        detail::field("name", account.display_name()),
        detail::field(
            "email",
            account.email.clone().unwrap_or_else(|| "-".to_owned()),
        ),
        detail::field(
            "username",
            account.username.clone().unwrap_or_else(|| "-".to_owned()),
        ),
        detail::field(
            "account id",
            account.id.clone().unwrap_or_else(|| "-".to_owned()),
        ),
        detail::blank(),
        detail::section("roles"),
        detail::field_highlighted("on this study", role.to_owned()),
        detail::field("platform role", account.role_label().to_owned()),
    ];

    frame.render_widget(detail::panel(block, lines), area);
}
