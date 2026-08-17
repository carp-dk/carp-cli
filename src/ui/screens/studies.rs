// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! The study list, the entry point of the app: the studies on the left, the
//! details of the highlighted one filling the larger panel on the right.

use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::text::Line;
use ratatui::widgets::Row;

use crate::app::App;
use crate::ui::widgets::{detail, master_detail, spinner, table};
use crate::ui::{icons, theme};
use carp_client::api::models::{StudyOverview, format_instant};

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    crate::app::state::clamp_selection(&mut app.studies.table, app.studies.visible.len());
    // The detail panel gets the larger share: the list only needs a name.
    let (list_area, detail_area) = master_detail(area, 2, 3);
    if let Some(detail_area) = detail_area {
        render_detail(frame, detail_area, app.studies.selected());
    }
    render_list(frame, list_area, app);
}

fn render_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let title = list_title(app);
    let block = theme::focused_block(&title);

    if app.studies.visible.is_empty() {
        let message = if app.studies.loading {
            "loading studies…"
        } else if app.studies.items.is_empty() {
            "no studies are visible to this account"
        } else {
            "no study matches the filter - press c to clear it"
        };
        frame.render_widget(table::placeholder(message, block), area);
        return;
    }

    let rows: Vec<Row> = app
        .studies
        .visible
        .iter()
        .filter_map(|index| app.studies.items.get(*index))
        .map(|study| {
            Row::new(vec![
                Line::raw(study.name.clone()),
                Line::styled(
                    icons::with(icons::study_stage(study.stage()), study.stage()),
                    stage_style(study),
                ),
                Line::raw(
                    study
                        .created_on
                        .map(|created| created.to_local_date())
                        .unwrap_or_else(|| "-".to_owned()),
                ),
            ])
        })
        .collect();

    let header = table::header(["Study", "Stage", "Created"]);
    let widths = vec![
        Constraint::Fill(1),
        Constraint::Length(10 + icons::cell_width()),
        Constraint::Length(10),
    ];

    let len = rows.len();
    table::render(
        frame,
        area,
        table::table(header, rows, widths, block),
        &mut app.studies.table,
        len,
    );
}

/// Everything the study list endpoint reports about one study. No extra
/// request is made while moving through the list; opening the study loads the
/// participants, staff and exports.
fn render_detail(frame: &mut Frame, area: Rect, study: Option<&StudyOverview>) {
    let block = theme::block("Study details");

    let Some(study) = study else {
        frame.render_widget(
            detail::empty(block, "select a study to see its details"),
            area,
        );
        return;
    };

    let lines = vec![
        detail::section("identity"),
        detail::field("name", study.name.clone()),
        detail::field("study id", study.study_id.to_string()),
        detail::field_styled(
            "stage",
            icons::with(icons::study_stage(study.stage()), study.stage()),
            stage_style(study),
        ),
        detail::blank(),
        detail::section("timeline"),
        detail::field("created", format_instant(study.created_on)),
        detail::field(
            "created by",
            study.created_by.clone().unwrap_or_else(|| "-".to_owned()),
        ),
        detail::blank(),
        detail::section("protocol"),
        detail::field(
            "protocol id",
            study
                .study_protocol_id
                .as_ref()
                .map_or_else(|| "not set".to_owned(), ToString::to_string),
        ),
        detail::blank(),
        detail::section("permissions"),
        detail::field("set invitation", yes_no(study.can_set_invitation)),
        detail::field("set protocol", yes_no(study.can_set_study_protocol)),
        detail::field("deploy", yes_no(study.can_deploy_to_participants)),
        detail::blank(),
        detail::section("description"),
        detail::text(study.description_line().to_owned()),
        detail::blank(),
        detail::note("enter opens the study · o opens it in the browser"),
    ];

    frame.render_widget(detail::panel(block, lines), area);
}

fn list_title(app: &App) -> String {
    let shown = app.studies.visible.len();
    let total = app.studies.items.len();
    let mut title = format!(
        "Studies {}",
        table::position_label(&app.studies.table, shown)
    );
    if shown != total {
        title.push_str(&format!(" of {total}"));
    }
    title.push_str(&format!(" · by {}", app.studies.sort.label()));
    if !app.studies.filter.trim().is_empty() {
        title.push_str(&format!(" · \"{}\"", app.studies.filter.trim()));
    }
    if app.studies.from_cache {
        title.push_str(" · cached");
    }
    let busy = spinner::label(app.ticks, app.studies.loading);
    if !busy.is_empty() {
        title.push_str(&format!(" · {busy}"));
    }
    title
}

fn stage_style(study: &StudyOverview) -> ratatui::style::Style {
    match study.stage() {
        "live" => theme::ok(),
        "configured" => theme::warn(),
        _ => theme::dim(),
    }
}

fn yes_no(value: bool) -> String {
    if value { "yes" } else { "no" }.to_owned()
}
