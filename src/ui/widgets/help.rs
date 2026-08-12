// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Full key reference, shown over the current screen.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, Padding, Paragraph};

use crate::ui::widgets::centered;
use crate::ui::{icons, theme};

/// Width of one column of keys.
const COLUMN_WIDTH: u16 = 42;

type Section = (&'static str, &'static [(&'static str, &'static str)]);

/// Icon for a help section, matched to the screen it describes.
fn section_icon(title: &str) -> &'static str {
    match title {
        "Studies" => icons::study(),
        "Study tabs" => icons::study(),
        "Participants" => icons::participants(),
        "Exports and files" => icons::exports(),
        "Downloads" => icons::downloads(),
        "Protocol editor" | "Protocol forms" => icons::app(),
        "Browser" => icons::app(),
        "Lists" => icons::help(),
        _ => icons::help(),
    }
}

const SECTIONS: [Section; 10] = [
    (
        "Global",
        &[
            ("q / ctrl-c", "quit"),
            ("esc", "back one screen"),
            ("r", "reload this view"),
            ("d", "downloads"),
            ("o", "open study in browser"),
            ("P", "protocol editor"),
            ("?", "toggle this help"),
        ],
    ),
    (
        "Lists",
        &[
            ("↑ ↓ / k j", "move; the panel follows"),
            ("pgup / pgdn", "jump ten rows"),
            ("ctrl-u / ctrl-d", "jump ten rows"),
            ("g / G", "first / last row"),
            ("enter", "open or download"),
        ],
    ),
    (
        "Studies",
        &[
            ("/", "filter as you type"),
            ("esc", "cancel, restore the list"),
            ("s", "sort: name, newest, stage"),
            ("c", "clear the filter"),
        ],
    ),
    (
        "Study tabs",
        &[
            ("tab / shift-tab", "next / previous tab"),
            ("← →", "next / previous tab"),
            ("1 … 6", "jump to a tab"),
        ],
    ),
    (
        "Participants",
        &[
            ("/", "search on the server"),
            ("n / p", "next / previous page"),
            ("s", "sort identity/deployed"),
            ("S", "reverse the sort"),
            ("f", "all / deployed / not yet"),
        ],
    ),
    (
        "Exports and files",
        &[
            ("enter", "download"),
            ("n", "request a data export"),
            ("x", "delete, after confirming"),
        ],
    ),
    (
        "Protocol editor",
        &[
            ("P", "open it; esc leaves"),
            ("tab / 1 … 8", "move between its tabs"),
            ("a / e / x", "add, edit, remove"),
            ("z", "undo the last change"),
            ("s / o / n", "save, open, new"),
            ("u", "upload to CARP"),
            ("v", "set the version tag"),
            ("S", "sync the upstream catalogue"),
        ],
    ),
    (
        "Protocol forms",
        &[
            ("enter", "open the selected field"),
            ("space", "flip a toggle or choice"),
            ("w", "save the form"),
            ("esc", "discard it"),
            ("type to filter", "in any picker"),
        ],
    ),
    (
        "Downloads",
        &[
            ("o", "open the saved folder"),
            ("c", "clear finished transfers"),
        ],
    ),
    (
        "Browser",
        &[
            ("o", "open the study in the portal"),
            ("", "already signed in via the browser"),
        ],
    ),
];

pub fn render(frame: &mut Frame, area: Rect) {
    // Two columns when there is room: the full list is too long for one on a
    // short terminal.
    let columns = if area.width >= COLUMN_WIDTH * 2 + 6 {
        2
    } else {
        1
    };
    let pages: Vec<Vec<Line>> = groups(columns)
        .into_iter()
        .map(|sections| sections.iter().flat_map(render_section).collect())
        .collect();

    let height = pages.iter().map(Vec::len).max().unwrap_or(0) as u16 + 2;
    let width = COLUMN_WIDTH * pages.len() as u16 + 4;
    let popup = centered(area, width, height);

    frame.render_widget(Clear, popup);
    frame.render_widget(
        theme::focused_block("Keys")
            .padding(Padding::horizontal(1))
            .title_bottom(Line::styled(" ? or esc closes ", theme::dim())),
        popup,
    );

    let inner = popup.inner(Margin {
        vertical: 1,
        horizontal: 2,
    });
    let areas = Layout::horizontal(vec![Constraint::Fill(1); pages.len()]).split(inner);
    for (lines, area) in pages.into_iter().zip(areas.iter()) {
        frame.render_widget(Paragraph::new(lines), *area);
    }
}

/// Split the sections across `columns`, keeping their order but balancing the
/// height so neither column runs off a short terminal.
fn groups(columns: usize) -> Vec<&'static [Section]> {
    if columns < 2 {
        return vec![&SECTIONS];
    }

    let heights: Vec<usize> = SECTIONS.iter().map(|(_, keys)| keys.len() + 2).collect();
    let total: usize = heights.iter().sum();

    let mut best = (usize::MAX, 1);
    let mut running = 0;
    for (index, height) in heights.iter().enumerate() {
        running += height;
        let difference = running.abs_diff(total - running);
        if difference < best.0 {
            best = (difference, index + 1);
        }
    }

    let (left, right) = SECTIONS.split_at(best.1);
    vec![left, right]
}

fn render_section(section: &Section) -> Vec<Line<'static>> {
    let (title, keys) = section;
    let mut lines = vec![Line::styled(
        icons::with(section_icon(title), *title),
        theme::title(),
    )];
    lines.extend(keys.iter().map(|(key, action)| {
        Line::from(vec![
            Span::styled(super::pad_to(key, 16), theme::table_header()),
            Span::styled((*action).to_owned(), theme::value()),
        ])
    }));
    lines.push(Line::raw(""));
    lines
}
