// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Table helpers so every list in the app looks and behaves the same.

use ratatui::Frame;
use ratatui::layout::{Constraint, Margin, Rect};
use ratatui::text::Line;
use ratatui::widgets::{
    Block, HighlightSpacing, Padding, Paragraph, Row, Scrollbar, ScrollbarOrientation,
    ScrollbarState, Table, TableState, Wrap,
};

use crate::ui::theme;

/// Rows a bordered table can show: two border lines and the header.
const CHROME_ROWS: u16 = 3;

/// Column titles.
pub fn header<'a, I, S>(labels: I) -> Row<'a>
where
    I: IntoIterator<Item = S>,
    S: Into<std::borrow::Cow<'a, str>>,
{
    Row::new(labels.into_iter().map(Into::into).collect::<Vec<_>>())
        .style(theme::table_header())
        .bottom_margin(0)
}

/// A selectable table with the app's styling.
pub fn table<'a>(
    header: Row<'a>,
    rows: Vec<Row<'a>>,
    widths: Vec<Constraint>,
    block: Block<'a>,
) -> Table<'a> {
    Table::new(rows, widths)
        .header(header)
        .block(block)
        .column_spacing(2)
        .row_highlight_style(theme::selected_row())
        .highlight_symbol(Line::styled("▌", theme::selection_bar()))
        .highlight_spacing(HighlightSpacing::Always)
}

/// Draw a table and, when it does not fit, the scrollbar that says so.
pub fn render<'a>(
    frame: &mut Frame,
    area: Rect,
    table: Table<'a>,
    state: &mut TableState,
    len: usize,
) {
    frame.render_stateful_widget(table, area, state);
    scrollbar(frame, area, len, state.selected().unwrap_or(0));
}

/// A scrollbar on the right border, drawn only when there is more to see.
///
/// Without it a long list looks identical to a short one, and there is no way
/// to tell where in the data the cursor is.
pub fn scrollbar(frame: &mut Frame, area: Rect, len: usize, position: usize) {
    let visible = area.height.saturating_sub(CHROME_ROWS) as usize;
    if len <= visible || area.height < CHROME_ROWS {
        return;
    }
    let mut state = ScrollbarState::new(len.saturating_sub(visible)).position(position);
    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .thumb_style(theme::label())
            .track_style(theme::dim()),
        area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut state,
    );
}

/// `12/48` for the block title, so the cursor position is always visible.
pub fn position_label(state: &TableState, len: usize) -> String {
    match state.selected() {
        Some(selected) if len > 0 => format!("{}/{len}", selected + 1),
        _ => len.to_string(),
    }
}

/// Placeholder for an empty or still-loading list.
pub fn placeholder<'a>(message: &'a str, block: Block<'a>) -> Paragraph<'a> {
    Paragraph::new(Line::styled(message, theme::dim()))
        .wrap(Wrap { trim: true })
        .centered()
        .block(block.padding(Padding::horizontal(1)))
}
