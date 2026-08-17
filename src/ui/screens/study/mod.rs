// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! One study, one module per tab. Every tab is a list on the left and a
//! detail panel for the highlighted row on the right, so the layout is
//! predictable wherever the user lands.

pub mod deployments;
pub mod exports;
pub mod files;
pub mod overview;
pub mod participants;
pub mod staff;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::Tabs;

use crate::app::App;
use crate::app::state::{StudyState, StudyTab};
use crate::ui::widgets::spinner;
use crate::ui::{icons, theme};

/// Proportions shared by every tab: the list leads, the detail supports.
pub const LIST_WEIGHT: u16 = 3;
pub const DETAIL_WEIGHT: u16 = 2;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    let ticks = app.ticks;
    let Some(study) = app.study.as_mut() else {
        return;
    };
    study.sync_selection();

    let [tabs_area, body] =
        Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);
    render_tabs(frame, tabs_area, study);

    match study.tab {
        StudyTab::Overview => overview::render(frame, body, study),
        StudyTab::Participants => participants::render(frame, body, study, ticks),
        StudyTab::Deployments => deployments::render(frame, body, study, ticks),
        StudyTab::Staff => staff::render(frame, body, study, ticks),
        StudyTab::Files => files::render(frame, body, study, ticks),
        StudyTab::Exports => exports::render(frame, body, study, ticks),
    }
}

fn render_tabs(frame: &mut Frame, area: Rect, study: &StudyState) {
    let titles: Vec<String> = StudyTab::ALL
        .iter()
        .enumerate()
        .map(|(index, tab)| {
            format!(
                " {} {}{} ",
                index + 1,
                icons::with(tab_icon(*tab), tab.title()),
                tab_count(study, *tab)
            )
        })
        .collect();

    let tabs = Tabs::new(titles)
        .select(study.tab.index())
        .style(theme::label())
        .highlight_style(theme::tab_selected())
        .divider("");
    frame.render_widget(tabs, area);
}

fn tab_icon(tab: StudyTab) -> &'static str {
    match tab {
        StudyTab::Overview => icons::study(),
        StudyTab::Participants => icons::participants(),
        StudyTab::Deployments => icons::deployments(),
        StudyTab::Staff => icons::staff(),
        StudyTab::Files => icons::files(),
        StudyTab::Exports => icons::exports(),
    }
}

/// How many rows a tab holds, once it has been loaded. Showing the number in
/// the tab bar answers "is there anything in there?" without a visit.
fn tab_count(study: &StudyState, tab: StudyTab) -> String {
    let count = match tab {
        StudyTab::Overview => None,
        StudyTab::Participants => study
            .participants
            .loaded
            .then_some(study.participants.total as usize),
        StudyTab::Deployments => study.details_loaded.then_some(study.groups().groups.len()),
        StudyTab::Staff => study.details_loaded.then(|| study.staff().len()),
        StudyTab::Files => study.files_loaded.then_some(study.files.len()),
        StudyTab::Exports => study.exports_loaded.then_some(study.exports.len()),
    };
    count.map(|count| format!(" {count}")).unwrap_or_default()
}

/// `Files 3/12 · ⠹ loading` - one title format for every tab.
pub fn tab_title(name: &str, count: impl Into<String>, ticks: usize, loading: bool) -> String {
    let mut title = format!("{name} {}", count.into());
    let busy = spinner::label(ticks, loading);
    if !busy.is_empty() {
        title.push_str(&format!(" · {busy}"));
    }
    title
}
