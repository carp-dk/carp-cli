// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Server-side data exports: request one, watch it build, download it.

use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::Row;

use crate::api::models::{Export, ExportStatus, format_instant};
use crate::app::state::StudyState;
use crate::ui::screens::study::{DETAIL_WEIGHT, LIST_WEIGHT, tab_title};
use crate::ui::widgets::{detail, master_detail, table};
use crate::ui::{icons, theme};

pub fn render(frame: &mut Frame, area: Rect, study: &mut StudyState, ticks: usize) {
    let (list_area, detail_area) = master_detail(area, LIST_WEIGHT, DETAIL_WEIGHT);
    if let Some(detail_area) = detail_area {
        render_detail(frame, detail_area, study.selected_export());
    }
    render_list(frame, list_area, study, ticks);
}

fn render_list(frame: &mut Frame, area: Rect, study: &mut StudyState, ticks: usize) {
    let title = tab_title(
        "Exports",
        table::position_label(&study.exports_table, study.exports.len()),
        ticks,
        study.exports_loading,
    );
    let block = theme::focused_block(&title);

    if study.exports.is_empty() {
        let message = if study.exports_loading {
            "loading exports…"
        } else {
            "no exports yet - press n to request one"
        };
        frame.render_widget(table::placeholder(message, block), area);
        return;
    }

    let rows: Vec<Row> = study
        .exports
        .iter()
        .map(|export| {
            Row::new(vec![
                // Never blank: a just-requested export has no archive name
                // until the server has packaged it.
                Line::raw(export.display_name()),
                Line::styled(
                    icons::with(status_icon(export.status), export.status.label()),
                    status_style(export.status),
                ),
                Line::raw(
                    export
                        .created_at
                        .map(|created| created.to_local_date())
                        .unwrap_or_else(|| "-".to_owned()),
                ),
            ])
        })
        .collect();

    let header = table::header(["Archive", "Status", "Created"]);
    let widths = vec![
        Constraint::Fill(1),
        Constraint::Length(12 + icons::cell_width()),
        Constraint::Length(10),
    ];

    let len = rows.len();
    table::render(
        frame,
        area,
        table::table(header, rows, widths, block),
        &mut study.exports_table,
        len,
    );
}

fn render_detail(frame: &mut Frame, area: Rect, export: Option<&Export>) {
    let block = theme::block("Export");

    let Some(export) = export else {
        frame.render_widget(
            detail::empty(
                block,
                "no exports selected - press n to request a study data export",
            ),
            area,
        );
        return;
    };

    let mut lines = vec![
        detail::section("archive"),
        detail::field("name", export.display_name()),
        detail::field("contents", export.kind.label().to_owned()),
        detail::field_styled(
            "status",
            icons::with(status_icon(export.status), export.status.label()),
            status_style(export.status),
        ),
        detail::blank(),
        detail::section("origin"),
        detail::field("export id", export.id.clone()),
        detail::field("created", format_instant(export.created_at)),
        detail::field(
            "created by",
            export.created_by.clone().unwrap_or_else(|| "-".to_owned()),
        ),
        detail::field("updated", format_instant(export.updated_at)),
        detail::blank(),
    ];

    // Say what the current status means for the user's next keystroke.
    lines.push(detail::note(match export.status {
        ExportStatus::Available => "Ready. Press enter to download it.",
        ExportStatus::InProgress => {
            "The server is still packaging this export. The list refreshes itself while it does; \
             the archive name appears once it is written."
        }
        ExportStatus::Error => {
            "The server failed to build this export. Press n to request a new one."
        }
        ExportStatus::Expired => "This export has expired. Press n to request a new one.",
        ExportStatus::Unknown => "The server did not report a status for this export.",
    }));

    if !export.relative_path.is_empty() {
        lines.push(detail::blank());
        lines.push(detail::section("server path"));
        lines.push(detail::note(export.relative_path.clone()));
    }

    lines.push(detail::blank());
    lines.push(detail::note("enter download · n request · x delete"));

    frame.render_widget(detail::panel(block, lines), area);
}

fn status_icon(status: ExportStatus) -> &'static str {
    match status {
        ExportStatus::Available => icons::ok(),
        ExportStatus::InProgress => icons::pending(),
        ExportStatus::Error => icons::error(),
        ExportStatus::Expired => icons::stopped(),
        ExportStatus::Unknown => icons::idle(),
    }
}

fn status_style(status: ExportStatus) -> Style {
    match status {
        ExportStatus::Available => theme::ok(),
        ExportStatus::InProgress => theme::warn(),
        ExportStatus::Error => theme::error(),
        _ => theme::dim(),
    }
}
