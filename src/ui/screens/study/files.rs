// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Files uploaded for the study, and where a download would put them.

use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::text::Line;
use ratatui::widgets::Row;

use crate::app::state::StudyState;
use crate::ui::screens::study::{DETAIL_WEIGHT, LIST_WEIGHT, tab_title};
use crate::ui::widgets::{detail, master_detail, table};
use crate::ui::{icons, theme};
use carp_client::api::models::{StudyFile, format_instant};

/// How much of a file's metadata document to show.
const METADATA_LINES: usize = 12;

pub fn render(frame: &mut Frame, area: Rect, study: &mut StudyState, ticks: usize) {
    let (list_area, detail_area) = master_detail(area, LIST_WEIGHT, DETAIL_WEIGHT);
    if let Some(detail_area) = detail_area {
        render_detail(frame, detail_area, study.selected_file());
    }
    render_list(frame, list_area, study, ticks);
}

fn render_list(frame: &mut Frame, area: Rect, study: &mut StudyState, ticks: usize) {
    let title = tab_title(
        "Files",
        table::position_label(&study.files_table, study.files.len()),
        ticks,
        study.files_loading,
    );
    let block = theme::focused_block(&title);

    if study.files.is_empty() {
        let message = if study.files_loading {
            "loading files…"
        } else {
            "no files have been uploaded for this study"
        };
        frame.render_widget(table::placeholder(message, block), area);
        return;
    }

    let rows: Vec<Row> = study
        .files
        .iter()
        .map(|file| {
            Row::new(vec![
                Line::raw(icons::with(icons::files(), file.download_name())),
                Line::raw(file.deployment_label().to_owned()),
                Line::raw(
                    file.created_at
                        .map(|created| created.to_local_date())
                        .unwrap_or_else(|| "-".to_owned()),
                ),
            ])
        })
        .collect();

    let header = table::header(["File", "Deployment", "Uploaded"]);
    let widths = vec![
        Constraint::Fill(1),
        Constraint::Length(12),
        Constraint::Length(10),
    ];

    let len = rows.len();
    table::render(
        frame,
        area,
        table::table(header, rows, widths, block),
        &mut study.files_table,
        len,
    );
}

fn render_detail(frame: &mut Frame, area: Rect, file: Option<&StudyFile>) {
    let block = theme::block("File");

    let Some(file) = file else {
        frame.render_widget(detail::empty(block, "no file selected"), area);
        return;
    };

    let mut lines = vec![
        detail::section("file"),
        detail::field("name", file.download_name().to_owned()),
        detail::field("stored as", file.file_name.clone()),
        detail::field("path", file.relative_path.clone()),
        detail::field("file id", file.id.to_string()),
        detail::blank(),
        detail::section("origin"),
        detail::field(
            "deployment",
            file.deployment_id.clone().unwrap_or_else(|| "-".to_owned()),
        ),
        detail::field(
            "owner",
            file.owner_id.clone().unwrap_or_else(|| "-".to_owned()),
        ),
        detail::field("uploaded", format_instant(file.created_at)),
        detail::field(
            "uploaded by",
            file.created_by.clone().unwrap_or_else(|| "-".to_owned()),
        ),
    ];

    if let Some(metadata) = &file.metadata
        && !metadata.is_null()
    {
        lines.push(detail::blank());
        lines.push(detail::section("metadata"));
        let rendered =
            serde_json::to_string_pretty(metadata).unwrap_or_else(|_| metadata.to_string());
        for line in rendered.lines().take(METADATA_LINES) {
            lines.push(detail::note(line.to_owned()));
        }
        if rendered.lines().count() > METADATA_LINES {
            lines.push(detail::note("…"));
        }
    }

    lines.push(detail::blank());
    lines.push(detail::note("enter downloads this file"));

    frame.render_widget(detail::panel(block, lines), area);
}
