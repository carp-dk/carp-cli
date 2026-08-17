// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! The detail panel that sits beside every list, and the line shapes it is
//! built from. Keeping them here is what makes the panels look identical
//! across screens.

use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Padding, Paragraph, Wrap};

use crate::ui::theme;

/// Width reserved for field labels, so values line up in a column.
const LABEL_WIDTH: usize = 16;

/// Pad `label` to the column, always leaving at least one space before the
/// value. A label exactly as wide as the column would otherwise run straight
/// into it - `informed consent` did.
fn label_column(label: &str) -> String {
    use unicode_width::UnicodeWidthStr;
    super::pad_to(label, LABEL_WIDTH.max(label.width() + 1))
}

/// Render `lines` inside `block`, wrapping long values.
pub fn panel<'a>(block: Block<'a>, lines: Vec<Line<'a>>) -> Paragraph<'a> {
    Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .block(block.padding(Padding::horizontal(1)))
}

/// Placeholder for when no row is selected.
pub fn empty<'a>(block: Block<'a>, message: &'a str) -> Paragraph<'a> {
    Paragraph::new(Line::styled(message, theme::dim()))
        .wrap(Wrap { trim: true })
        .centered()
        .block(block.padding(Padding::horizontal(1)))
}

/// Heading above a group of fields.
pub fn section(title: &str) -> Line<'static> {
    Line::styled(title.to_owned(), theme::title())
}

pub fn blank() -> Line<'static> {
    Line::raw("")
}

/// `label   value`.
pub fn field(label: &str, value: impl Into<String>) -> Line<'static> {
    Line::from(vec![
        Span::styled(label_column(label), theme::label()),
        Span::styled(value.into(), theme::value()),
    ])
}

/// Same, with the value picked out in the accent colour.
pub fn field_highlighted(label: &str, value: impl Into<String>) -> Line<'static> {
    Line::from(vec![
        Span::styled(label_column(label), theme::label()),
        Span::styled(
            value.into(),
            ratatui::style::Style::new().fg(theme::HIGHLIGHT),
        ),
    ])
}

/// Same, with a caller-chosen style for the value (state colouring).
pub fn field_styled(
    label: &str,
    value: impl Into<String>,
    style: ratatui::style::Style,
) -> Line<'static> {
    Line::from(vec![
        Span::styled(label_column(label), theme::label()),
        Span::styled(value.into(), style),
    ])
}

/// Free text at value emphasis.
pub fn text(value: impl Into<String>) -> Line<'static> {
    Line::styled(value.into(), theme::value())
}

/// Secondary text: hints, explanations, key reminders.
pub fn note(value: impl Into<String>) -> Line<'static> {
    Line::styled(value.into(), theme::dim())
}

/// A bullet inside a section, e.g. one device of a deployment.
pub fn bullet(value: impl Into<String>, style: ratatui::style::Style) -> Line<'static> {
    Line::from(vec![
        Span::styled("  • ", theme::dim()),
        Span::styled(value.into(), style),
    ])
}
