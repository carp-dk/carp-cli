// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Colours and styles, kept in one place so the screens stay consistent.
//!
//! Every colour is a fixed 256-colour palette index rather than an ANSI name.
//! Terminals re-theme ANSI 0-15 freely, so `Color::Red` is a hard fire-engine
//! red in one terminal and a soft pink in another; indices 16-255 are the
//! standard xterm colour cube and render the same everywhere. That keeps the
//! app looking like itself in Ghostty, Zed, iTerm and tmux alike.
//!
//! The palette is deliberately desaturated: a terminal UI is read for minutes
//! at a time, and full-intensity red on black is tiring.

use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, BorderType};

/// Soft coral: the CARP red, toned down for long reading (`#ff8787`).
pub const ACCENT: Color = Color::Indexed(210);
/// Dusty rose for larger areas of accent, such as a focused border
/// (`#d78787`).
pub const ACCENT_MUTED: Color = Color::Indexed(174);
/// Sky blue for values worth pointing at (`#87d7ff`).
pub const HIGHLIGHT: Color = Color::Indexed(117);

/// Something is fine or finished (`#87d787`).
const GOOD: Color = Color::Indexed(114);
/// Something is pending or needs attention (`#ffd787`).
const CAUTION: Color = Color::Indexed(222);
/// Something failed. The one place a saturated red is warranted (`#ff5f5f`).
const BAD: Color = Color::Indexed(203);

/// Primary text (`#d0d0d0`), softer than pure white.
const TEXT: Color = Color::Indexed(252);
/// Labels and secondary text (`#8a8a8a`).
const MUTED: Color = Color::Indexed(245);
/// Borders, hints, anything that should recede (`#585858`).
const FAINT: Color = Color::Indexed(240);
/// Background of the selected row (`#3a3a3a`).
const SELECTION: Color = Color::Indexed(237);
/// Text on top of an accent fill (`#262626`).
const ON_ACCENT: Color = Color::Indexed(235);

/// The ` CARP ` badge at the far left of the header.
pub fn badge() -> Style {
    Style::new().fg(ON_ACCENT).bg(ACCENT).bold()
}

/// Rest of the header line: present, but never competing with the content.
pub fn header() -> Style {
    Style::new().fg(MUTED)
}

/// The breadcrumb showing where the user is.
pub fn breadcrumb() -> Style {
    Style::new().fg(ACCENT)
}

pub fn title() -> Style {
    Style::new().fg(ACCENT).bold()
}

pub fn table_header() -> Style {
    Style::new().fg(HIGHLIGHT).add_modifier(Modifier::BOLD)
}

/// The highlighted row of the focused list.
pub fn selected_row() -> Style {
    Style::new().fg(TEXT).bg(SELECTION).bold()
}

/// The `▌` bar marking the selected row.
pub fn selection_bar() -> Style {
    Style::new().fg(ACCENT).bg(SELECTION)
}

pub fn label() -> Style {
    Style::new().fg(MUTED)
}

pub fn value() -> Style {
    Style::new().fg(TEXT)
}

pub fn dim() -> Style {
    Style::new().fg(FAINT)
}

pub fn ok() -> Style {
    Style::new().fg(GOOD)
}

pub fn warn() -> Style {
    Style::new().fg(CAUTION)
}

pub fn error() -> Style {
    Style::new().fg(BAD).bold()
}

/// Selected tab in the study tab bar.
pub fn tab_selected() -> Style {
    Style::new().fg(ON_ACCENT).bg(ACCENT).bold()
}

/// Rounded block with a coloured title, used by every panel.
pub fn block(title: &str) -> Block<'_> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(dim())
        .title(ratatui::text::Span::styled(
            format!(" {title} "),
            self::title(),
        ))
}

/// Block for the panel that currently has the keyboard. The border uses the
/// muted accent: a full-strength outline around a tall panel is a lot of
/// colour for something that only needs to say "focus is here".
pub fn focused_block(title: &str) -> Block<'_> {
    block(title).border_style(Style::new().fg(ACCENT_MUTED))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ANSI colours 0-15 are what terminals re-theme, which is why the same
    /// red came out harsh in one terminal and soft in another. Every style
    /// here must therefore use the fixed part of the 256-colour palette.
    #[test]
    fn no_style_depends_on_terminal_theming() {
        let styles = [
            badge(),
            header(),
            breadcrumb(),
            title(),
            table_header(),
            selected_row(),
            selection_bar(),
            label(),
            value(),
            dim(),
            ok(),
            warn(),
            error(),
            tab_selected(),
        ];

        for style in styles {
            for color in [style.fg, style.bg].into_iter().flatten() {
                assert!(
                    matches!(color, Color::Indexed(index) if index >= 16),
                    "{color:?} is an ANSI colour and will be re-themed per terminal"
                );
            }
        }
    }
}
