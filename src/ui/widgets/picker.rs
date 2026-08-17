// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Rendering a [`Picker`] as a centred overlay.
//!
//! A filter box at the top, the matching rows beneath, and the keys at the
//! bottom. Each row shows its value and, dimmed beside it, what the value
//! means or how widely it is used - the second column is why the measure-type
//! picker is usable at all, since the values themselves differ only in their
//! last segment.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, Paragraph, Row};

use crate::app::form::picker::{Picker, PickerKind};
use crate::ui::theme;
use crate::ui::widgets::{centered, table};

/// The overlay's size, as a fraction of the screen it will not exceed.
const MAX_WIDTH: u16 = 88;
const MAX_HEIGHT: u16 = 26;

/// Draw `picker` centred over `area`.
pub fn render(frame: &mut Frame, area: Rect, picker: &mut Picker) {
    let width = MAX_WIDTH.min(area.width.saturating_sub(4));
    let height = MAX_HEIGHT.min(area.height.saturating_sub(2));
    let overlay = centered(area, width, height);

    frame.render_widget(Clear, overlay);

    // The filter line sits above the list rather than inside its block, so
    // the list keeps a full border to scroll within.
    let [filter_area, list_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(overlay);

    frame.render_widget(Paragraph::new(filter_line(picker)), filter_area);
    render_rows(frame, list_area, picker);
}

/// `search ▸ <what was typed>`, with a count of what it matched.
fn filter_line(picker: &Picker) -> Line<'static> {
    let mut spans = vec![
        Span::styled(format!(" {} ", picker.title), theme::badge()),
        Span::raw(" "),
    ];

    if picker.filter.is_empty() {
        spans.push(Span::styled("type to filter", theme::dim()));
    } else {
        spans.push(Span::styled(picker.filter.clone(), theme::value()));
        spans.push(Span::styled("█", theme::label()));
    }

    spans.push(Span::styled(
        format!("  {} of {}", picker.visible.len(), picker.rows.len()),
        theme::dim(),
    ));
    Line::from(spans)
}

/// The matching rows, or a note saying nothing matched.
fn render_rows(frame: &mut Frame, area: Rect, picker: &mut Picker) {
    let multiple = picker.kind == PickerKind::Multiple;
    let heading = title(picker);
    let block = theme::focused_block(&heading);

    if picker.visible.is_empty() {
        let message = if picker.accepts_free_text {
            "nothing matches - Enter uses what you typed"
        } else {
            "nothing matches"
        };
        frame.render_widget(table::placeholder(message, block), area);
        return;
    }

    let rows: Vec<Row> = picker
        .visible
        .iter()
        .filter_map(|index| picker.rows.get(*index))
        .map(|row| {
            // A tick column only appears for a multi-select, where it is the
            // only way to see what is already chosen.
            let mark = if multiple {
                if picker.is_chosen(&row.value) {
                    "✓ "
                } else {
                    "  "
                }
            } else {
                ""
            };
            Row::new(vec![
                Line::styled(format!("{mark}{}", row.label), theme::value()),
                Line::styled(row.detail.clone(), theme::dim()),
            ])
        })
        .collect();

    let widths = vec![Constraint::Percentage(45), Constraint::Percentage(55)];
    let table = table::table(table::header(["value", "about"]), rows, widths, block);

    let mut state = ratatui::widgets::TableState::default().with_selected(Some(picker.selected));
    table::render(frame, area, table, &mut state, picker.visible.len());
}

/// The block title carries the keys, since a picker has no room for a footer.
fn title(picker: &Picker) -> String {
    let keys = match picker.kind {
        PickerKind::Multiple => "Space tick · Enter done · Esc cancel",
        _ => "Enter choose · Esc cancel",
    };
    format!("{keys} ")
}

#[cfg(test)]
mod tests;
