// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Transfer queue and the log of everything downloaded before.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::Row;

use crate::app::App;
use crate::download::JobStatus;
use crate::ui::widgets::table;
use crate::ui::{icons, theme};
use carp_client::api::models::format_bytes;

/// Width of the textual progress bar.
const BAR_WIDTH: usize = 20;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    crate::app::state::clamp_selection(&mut app.downloads_table, app.downloads.jobs().len());
    let [active_area, history_area] =
        Layout::vertical([Constraint::Fill(3), Constraint::Fill(2)]).areas(area);

    render_active(frame, active_area, app);
    render_history(frame, history_area, app);
}

fn render_active(frame: &mut Frame, area: Rect, app: &mut App) {
    let title = format!(
        "Transfers {} · {} active · {}",
        table::position_label(&app.downloads_table, app.downloads.jobs().len()),
        app.downloads.active_count(),
        app.config.download_dir.display()
    );
    let block = theme::focused_block(&title);

    if app.downloads.is_empty() {
        frame.render_widget(
            table::placeholder(
                "nothing downloading yet - press enter on a study file or an available export",
                block,
            ),
            area,
        );
        return;
    }

    let rows: Vec<Row> = app
        .downloads
        .jobs()
        .iter()
        .map(|job| {
            let (status, style) = match &job.status {
                JobStatus::Running => (icons::with(icons::pending(), "running"), theme::warn()),
                JobStatus::Done => (icons::with(icons::ok(), "done"), theme::ok()),
                JobStatus::Failed(_) => (icons::with(icons::error(), "failed"), theme::error()),
            };
            Row::new(vec![
                Line::raw(job.label.clone()),
                Line::styled(bar(job.ratio()), theme::table_header()),
                Line::raw(job.progress_label()),
                Line::from(Span::styled(status, style)),
                Line::styled(
                    job.path
                        .as_ref()
                        .map(|path| path.display().to_string())
                        .unwrap_or_default(),
                    theme::dim(),
                ),
            ])
        })
        .collect();

    let header = table::header(["What", "Progress", "Size", "Status", "Saved to"]);
    let widths = vec![
        Constraint::Fill(2),
        Constraint::Length(BAR_WIDTH as u16 + 2),
        Constraint::Length(18),
        Constraint::Length(9 + icons::cell_width()),
        Constraint::Fill(3),
    ];

    let len = rows.len();
    table::render(
        frame,
        area,
        table::table(header, rows, widths, block),
        &mut app.downloads_table,
        len,
    );
}

fn render_history(frame: &mut Frame, area: Rect, app: &App) {
    let block = theme::block("Earlier downloads");

    if app.history.is_empty() {
        frame.render_widget(table::placeholder("no downloads recorded yet", block), area);
        return;
    }

    let rows: Vec<Row> = app
        .history
        .iter()
        .map(|record| {
            Row::new(vec![
                icons::with(icons::downloads(), &record.label),
                format_bytes(record.bytes),
                record
                    .finished_at
                    .split('T')
                    .next()
                    .unwrap_or("")
                    .to_owned(),
                record.path.display().to_string(),
            ])
        })
        .collect();

    let header = table::header(["What", "Size", "When", "Path"]);
    let widths = vec![
        Constraint::Fill(2),
        Constraint::Length(10),
        Constraint::Length(12),
        Constraint::Fill(3),
    ];

    frame.render_widget(table::table(header, rows, widths, block), area);
}

/// `[########------------]`, or an indeterminate bar without a known total.
fn bar(ratio: Option<f64>) -> String {
    match ratio {
        Some(ratio) => {
            let filled = (ratio * BAR_WIDTH as f64).round() as usize;
            format!(
                "[{}{}]",
                "█".repeat(filled.min(BAR_WIDTH)),
                "░".repeat(BAR_WIDTH - filled.min(BAR_WIDTH))
            )
        }
        None => format!("[{}]", "░".repeat(BAR_WIDTH)),
    }
}
