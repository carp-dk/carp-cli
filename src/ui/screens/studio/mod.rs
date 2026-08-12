// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Rendering the protocol editor.
//!
//! A tab bar, the current section, and the overlays on top. Each section is
//! its own module and follows the same shape as the rest of the app: the list
//! on the left, the panel describing the highlighted row on the right.

pub mod catalog;
pub mod checks;
pub mod devices;
pub mod overview;
pub mod participants;
pub mod survey;
pub mod tasks;
pub mod triggers;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::studio::{Section, Studio};
use crate::ui::theme;
use crate::ui::widgets::{form, picker};

/// Draw the editor into `area`.
pub fn render(frame: &mut Frame, area: Rect, studio: &mut Studio) {
    let [tabs_area, body_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);

    render_tabs(frame, tabs_area, studio);

    match studio.section {
        Section::Overview => overview::render(frame, body_area, studio),
        Section::Devices => devices::render(frame, body_area, studio),
        Section::Tasks => tasks::render(frame, body_area, studio),
        Section::Triggers => triggers::render(frame, body_area, studio),
        Section::Survey => survey::render(frame, body_area, studio),
        Section::Participants => participants::render(frame, body_area, studio),
        Section::Catalog => catalog::render(frame, body_area, studio),
        Section::Checks => checks::render(frame, body_area, studio),
    }

    // The form draws first so a picker opened from it lands on top, which is
    // the order they are dismissed in.
    if let Some(open) = studio.form.as_ref() {
        form::render(frame, body_area, open);
    }
    if let Some(open) = studio.picker.as_mut() {
        picker::render(frame, body_area, open);
    }
}

/// The tab bar, with a check summary on the right.
///
/// The summary is on the tab bar rather than only in the Checks tab because
/// an error that appears while editing devices should be visible from the
/// devices tab.
fn render_tabs(frame: &mut Frame, area: Rect, studio: &Studio) {
    let mut spans = Vec::with_capacity(Section::ALL.len() * 2);
    for section in Section::ALL {
        let style = if section == studio.section {
            theme::tab_selected()
        } else {
            theme::label()
        };
        spans.push(Span::styled(format!(" {} ", section.title()), style));
        spans.push(Span::raw(" "));
    }

    let [left, right] =
        Layout::horizontal([Constraint::Fill(3), Constraint::Fill(1)]).areas(area);
    frame.render_widget(Paragraph::new(Line::from(spans)), left);
    frame.render_widget(
        Paragraph::new(check_summary(studio)).right_aligned(),
        right,
    );
}

/// `2 errors · 1 warning`, or a tick when the protocol is sound.
fn check_summary(studio: &Studio) -> Line<'static> {
    let (errors, warnings, _) = studio.check_counts();

    if errors == 0 && warnings == 0 {
        return Line::styled("✓ no findings ", theme::ok());
    }

    let mut spans = Vec::new();
    if errors > 0 {
        spans.push(Span::styled(
            format!("{errors} error{} ", plural(errors)),
            theme::error(),
        ));
    }
    if warnings > 0 {
        spans.push(Span::styled(
            format!("{warnings} warning{} ", plural(warnings)),
            theme::warn(),
        ));
    }
    Line::from(spans)
}

fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}

/// The line of key hints for the current section, shown in the status bar.
pub fn hints(studio: &Studio) -> String {
    if studio.picker.is_some() {
        return "Enter choose · Esc cancel · type to filter".to_owned();
    }
    if studio.form.is_some() {
        return "↑↓ move · Enter open · w save · Esc discard".to_owned();
    }
    format!("{} · z undo · Esc leave", studio.section.hints())
}

#[cfg(test)]
mod dump;
#[cfg(test)]
mod tests;
