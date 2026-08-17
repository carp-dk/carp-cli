// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Rendering an editing [`Form`] as a centred overlay.
//!
//! The layout is a label column and a value column, one row per field, with
//! three lines beneath: the selected field's explanation, the error if the
//! last commit was refused, and the keys. Those three are always present -
//! reserving the space rather than growing into it means the rows do not jump
//! when an error appears.
//!
//! The field being typed into shows a cursor and its buffer rather than the
//! stored value, which is the only visual difference between the two modes and
//! so has to be unmistakable.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, Paragraph, Wrap};

use crate::app::form::{Form, Mode};
use crate::ui::theme;
use crate::ui::widgets::{centered, pad_to};

/// Width reserved for the label column.
const LABEL_WIDTH: usize = 22;
/// Lines of chrome: two borders, the three footer lines and their separator.
const CHROME: u16 = 7;
/// The overlay never grows past this, so a long form scrolls the terminal
/// rather than filling it edge to edge.
const MAX_WIDTH: u16 = 84;

/// Draw `form` centred over `area`.
pub fn render(frame: &mut Frame, area: Rect, form: &Form) {
    let height = (form.fields.len() as u16 + CHROME).min(area.height);
    let width = MAX_WIDTH.min(area.width.saturating_sub(4));
    let overlay = centered(area, width, height);

    frame.render_widget(Clear, overlay);
    frame.render_widget(theme::focused_block(&form.subject.title()), overlay);

    let inner = overlay.inner(ratatui::layout::Margin {
        vertical: 1,
        horizontal: 2,
    });
    if inner.height == 0 {
        return;
    }

    // Fields on top, the three-line footer at the bottom.
    let [rows, footer] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(3)]).areas(inner);

    frame.render_widget(Paragraph::new(field_lines(form)), rows);
    frame.render_widget(
        Paragraph::new(footer_lines(form)).wrap(Wrap { trim: true }),
        footer,
    );
}

/// One line per field: `label   value`, with the selected one marked.
fn field_lines(form: &Form) -> Vec<Line<'static>> {
    form.fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let selected = index == form.selected;
            let marker = if selected { "▌ " } else { "  " };

            let value = if selected && let Mode::Typing { buffer } = &form.mode {
                // A visible cursor is what separates typing from browsing.
                format!("{buffer}█")
            } else {
                field.value.display()
            };

            let value_style = match (selected, form.is_typing()) {
                (true, true) => Style::new().fg(theme::HIGHLIGHT),
                (true, false) => theme::selected_row(),
                _ => theme::value(),
            };

            Line::from(vec![
                Span::styled(marker, theme::selection_bar()),
                Span::styled(pad_to(&field.label, LABEL_WIDTH), theme::label()),
                Span::styled(value, value_style),
            ])
        })
        .collect()
}

/// Help for the selected field, the last error, and the keys.
fn footer_lines(form: &Form) -> Vec<Line<'static>> {
    let help = form
        .selected_field()
        .map(|field| field.help.clone())
        .unwrap_or_default();

    let error = match &form.error {
        Some(error) => Line::styled(error.clone(), theme::error()),
        None => Line::raw(""),
    };

    vec![
        Line::styled(help, theme::dim()),
        error,
        Line::styled(keys(form), theme::dim()),
    ]
}

/// The keys that apply right now. They differ between the two modes, and
/// showing the wrong set is worse than showing none.
fn keys(form: &Form) -> &'static str {
    if form.is_typing() {
        "Enter accept · Esc cancel · Ctrl-U clear"
    } else {
        "↑↓ move · Enter open · Space toggle · w save · Esc discard"
    }
}

#[cfg(test)]
mod tests;
