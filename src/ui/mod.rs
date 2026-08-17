// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Rendering. `draw` is the only entry point: header, body for the current
//! route, status bar, and the help overlay on top.

pub mod icons;
pub mod screens;
pub mod theme;
pub mod widgets;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Padding, Paragraph};

use crate::app::App;
use crate::app::state::Route;

/// Below this the layout stops being usable, so say so instead of drawing
/// something broken.
const MIN_WIDTH: u16 = 62;
const MIN_HEIGHT: u16 = 14;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        render_too_small(frame, area);
        return;
    }

    let [header_area, body_area, status_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(area);

    render_header(frame, header_area, app);

    match app.route {
        Route::Studies => screens::studies::render(frame, body_area, app),
        Route::Study => screens::study::render(frame, body_area, app),
        Route::Participant => {
            if let Some(participant) = app.participant.as_ref() {
                screens::participant::render(frame, body_area, participant);
            }
        }
        Route::Downloads => screens::downloads::render(frame, body_area, app),
        Route::Studio => {
            if let Some(studio) = app.studio.as_mut() {
                screens::studio::render(frame, body_area, studio);
            }
        }
    }

    widgets::status_bar::render(frame, status_area, app);

    if app.show_help {
        widgets::help::render(frame, body_area);
    }
}

fn render_too_small(frame: &mut Frame, area: Rect) {
    let lines = vec![
        Line::styled("terminal too small", theme::error()),
        Line::styled(
            format!(
                "{MIN_WIDTH}x{MIN_HEIGHT} needed, {}x{} now",
                area.width, area.height
            ),
            theme::dim(),
        ),
    ];
    frame.render_widget(
        Paragraph::new(lines)
            .centered()
            .block(Block::new().padding(Padding::top(area.height.saturating_sub(2) / 2))),
        area,
    );
}

/// `CARP · host · account · activity` on the left, the breadcrumb on the
/// right. The badge is the only place the brand colour fills a background.
fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let mut left = vec![
        Span::styled(
            format!(" {} ", icons::with(icons::app(), "CARP")),
            theme::badge(),
        ),
        Span::styled(format!(" {}", host(app.client.server())), theme::header()),
    ];
    if let Some(account) = &app.account {
        left.push(Span::styled(" · ", theme::dim()));
        left.push(Span::styled(account.clone(), theme::header()));
    }
    if app.is_busy() {
        left.push(Span::styled(" · ", theme::dim()));
        left.push(Span::styled(
            widgets::spinner::frame(app.ticks).to_owned(),
            theme::warn(),
        ));
    }
    let active = app.downloads.active_count();
    if active > 0 {
        left.push(Span::styled(" · ", theme::dim()));
        left.push(Span::styled(
            format!(
                "{} {active} downloading",
                widgets::spinner::frame(app.ticks)
            ),
            Style::new().fg(theme::HIGHLIGHT),
        ));
    }

    let [left_area, right_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).areas(area);
    frame.render_widget(Paragraph::new(Line::from(left)), left_area);
    frame.render_widget(
        Paragraph::new(Line::styled(breadcrumb(app), theme::breadcrumb())).right_aligned(),
        right_area,
    );
}

/// `https://dev.carp.dk/` reads as `dev.carp.dk` in a header.
fn host(server: &str) -> String {
    server
        .trim_end_matches('/')
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .to_owned()
}

fn breadcrumb(app: &App) -> String {
    let mut parts = vec!["studies".to_owned()];
    if let Some(study) = app.study.as_ref() {
        parts.push(study.study.name.clone());
        if app.route == Route::Study {
            parts.push(study.tab.title().to_lowercase());
        }
    }
    if let Some(participant) = app.participant.as_ref()
        && app.route == Route::Participant
    {
        parts.push(participant.participant.display_name());
    }
    if app.route == Route::Downloads {
        parts.push("downloads".to_owned());
    }
    if app.route == Route::Studio
        && let Some(studio) = app.studio.as_ref()
    {
        parts = vec![
            "protocol".to_owned(),
            studio.location(),
            studio.section.title().to_lowercase(),
        ];
    }
    format!("{} ", parts.join(" › "))
}

#[cfg(test)]
mod dump;
#[cfg(test)]
mod tests;
