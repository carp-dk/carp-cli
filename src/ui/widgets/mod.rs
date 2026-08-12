// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Small building blocks shared by the screens.

pub mod detail;
pub mod form;
pub mod help;
pub mod picker;
pub mod spinner;
pub mod status_bar;
pub mod table;

use ratatui::layout::{Constraint, Layout, Rect};
use unicode_width::UnicodeWidthStr;

/// A detail panel narrower than this cannot hold a label and a value, so it is
/// dropped rather than shown squeezed.
pub const MIN_DETAIL_WIDTH: u16 = 34;
/// A list narrower than this truncates the names it exists to show.
pub const MIN_LIST_WIDTH: u16 = 44;

/// Split a screen into a list and the panel describing its selected row.
///
/// Every list in the app uses this, so the eye always finds the detail in the
/// same place. `list` and `detail` are relative weights. On a narrow terminal
/// the panel is dropped and the list takes the full width: a cramped table
/// next to a cramped panel helps nobody.
pub fn master_detail(area: Rect, list: u16, detail: u16) -> (Rect, Option<Rect>) {
    let [list_area, detail_area] =
        Layout::horizontal([Constraint::Fill(list), Constraint::Fill(detail)]).areas(area);
    // Two cramped panes are worse than one usable one.
    if detail_area.width < MIN_DETAIL_WIDTH || list_area.width < MIN_LIST_WIDTH {
        (area, None)
    } else {
        (list_area, Some(detail_area))
    }
}

/// Pad `text` to `width` terminal cells.
///
/// Not `{:<width$}`: that counts characters, and an emoji is one character
/// occupying two cells, which would shift everything after it by one.
pub fn pad_to(text: &str, width: usize) -> String {
    let padding = width.saturating_sub(text.width());
    format!("{text}{}", " ".repeat(padding))
}

/// A `width` x `height` rectangle centred in `area`, clamped to fit.
pub fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + (area.height - height) / 2,
        width,
        height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bug emoji icons exposed: an emoji is one character but two cells,
    /// so character-counted padding shifted every value after it.
    #[test]
    fn padding_counts_cells_not_characters() {
        assert_eq!(pad_to("participants", 16).width(), 16);
        assert_eq!(pad_to("👥 participants", 16).width(), 16);
        assert_eq!(pad_to("◍ participants", 16).width(), 16);
        // Something already too wide is left alone rather than truncated.
        assert_eq!(pad_to("a very long label indeed", 16).width(), 24);
    }
}
