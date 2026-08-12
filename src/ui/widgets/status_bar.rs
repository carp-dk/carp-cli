// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Bottom line: the open prompt, the last status message, or the key hints
//! for the current screen. Exactly one of the three, so the line never
//! competes with itself for attention.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::App;
use crate::app::state::{Prompt, Route, StatusKind, StudyTab};
use crate::ui::{icons, theme};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    if let Some(prompt) = &app.prompt {
        render_prompt(frame, area, prompt);
        return;
    }

    if let Some(status) = &app.status {
        let style = match status.kind {
            StatusKind::Info => theme::ok(),
            StatusKind::Error => theme::error(),
        };
        let marker = match status.kind {
            StatusKind::Info => icons::ok(),
            StatusKind::Error => icons::error(),
        };
        let line = Line::from(vec![Span::styled(icons::with(marker, &status.text), style)]);
        frame.render_widget(Paragraph::new(line), area);
        return;
    }

    frame.render_widget(Paragraph::new(hints(app)), area);
}

fn render_prompt(frame: &mut Frame, area: Rect, prompt: &Prompt) {
    let label = prompt.label();

    if prompt.is_confirmation() {
        let line = Line::from(vec![
            Span::styled(format!("{label} "), theme::warn()),
            Span::styled("y", theme::table_header()),
            Span::styled(" confirm · ", theme::dim()),
            Span::styled("n", theme::table_header()),
            Span::styled(" cancel", theme::dim()),
        ]);
        frame.render_widget(Paragraph::new(line), area);
        return;
    }

    let prefix = format!("{label}: ");
    let line = Line::from(vec![
        Span::styled(prefix.clone(), theme::title()),
        Span::styled(prompt.value.clone(), theme::value()),
        Span::styled("   enter apply · esc cancel", theme::dim()),
    ]);
    frame.render_widget(Paragraph::new(line), area);

    // A real terminal cursor beats a drawn one: it blinks where the next
    // keystroke will land.
    let column = (prefix.chars().count() + prompt.value.chars().count()) as u16;
    if column < area.width {
        frame.set_cursor_position((area.x + column, area.y));
    }
}

/// Key hints for whatever is on screen.
fn hints(app: &App) -> Line<'static> {
    // The editor's hints change per tab and per overlay, so it composes its
    // own line rather than being enumerated here.
    if app.route == Route::Studio
        && let Some(studio) = app.studio.as_ref()
    {
        return Line::styled(
            crate::ui::screens::studio::hints(studio),
            theme::dim(),
        );
    }

    let keys: Vec<(&str, &str)> = match app.route {
        Route::Studies => vec![
            ("↑↓", "move"),
            ("enter", "open"),
            ("/", "filter"),
            ("s", "sort"),
            ("o", "browser"),
            ("P", "protocols"),
            ("r", "refresh"),
            ("d", "downloads"),
            ("?", "help"),
            ("q", "quit"),
        ],
        Route::Study => {
            let tab = app.study.as_ref().map_or(StudyTab::Overview, |s| s.tab);
            let mut keys = vec![("tab", "next tab"), ("1-6", "tab"), ("o", "browser")];
            match tab {
                StudyTab::Overview | StudyTab::Staff | StudyTab::Deployments => {}
                StudyTab::Participants => keys.extend([
                    ("enter", "open"),
                    ("/", "search"),
                    ("n/p", "page"),
                    ("s/S", "sort"),
                    ("f", "filter"),
                ]),
                StudyTab::Files => keys.push(("enter", "download")),
                StudyTab::Exports => {
                    keys.extend([("enter", "download"), ("n", "new export"), ("x", "delete")]);
                }
            }
            keys.extend([("esc", "back"), ("?", "help"), ("q", "quit")]);
            keys
        }
        Route::Participant => vec![("esc", "back"), ("?", "help"), ("q", "quit")],
        // Unreachable: handled above, before this match.
        Route::Studio => Vec::new(),
        Route::Downloads => vec![
            ("↑↓", "move"),
            ("o", "open folder"),
            ("c", "clear finished"),
            ("esc", "back"),
            ("q", "quit"),
        ],
    };

    let mut spans = Vec::new();
    for (index, (key, action)) in keys.into_iter().enumerate() {
        if index > 0 {
            spans.push(Span::styled(" · ", theme::dim()));
        }
        spans.push(Span::styled(key.to_owned(), theme::table_header()));
        spans.push(Span::styled(format!(" {action}"), theme::dim()));
    }
    Line::from(spans)
}
