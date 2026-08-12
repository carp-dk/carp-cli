// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Study metadata, and a summary of what the other tabs hold.

use ratatui::Frame;
use ratatui::layout::Rect;

use crate::api::models::format_instant;
use crate::app::state::StudyState;
use crate::ui::screens::study::{DETAIL_WEIGHT, LIST_WEIGHT};
use crate::ui::widgets::{detail, master_detail};
use crate::ui::{icons, theme};

pub fn render(frame: &mut Frame, area: Rect, study: &StudyState) {
    let (left, right) = master_detail(area, LIST_WEIGHT, DETAIL_WEIGHT);
    render_study(frame, left, study);
    if let Some(right) = right {
        render_glance(frame, right, study);
    }
}

fn render_study(frame: &mut Frame, area: Rect, study: &StudyState) {
    let overview = &study.study;
    let lines = vec![
        detail::section("identity"),
        detail::field("name", overview.name.clone()),
        detail::field("study id", overview.study_id.to_string()),
        detail::field_highlighted(
            "stage",
            icons::with(icons::study_stage(overview.stage()), overview.stage()),
        ),
        detail::blank(),
        detail::section("timeline"),
        detail::field("created", format_instant(overview.created_on)),
        detail::field(
            "created by",
            overview
                .created_by
                .clone()
                .unwrap_or_else(|| "-".to_owned()),
        ),
        detail::blank(),
        detail::section("protocol"),
        detail::field(
            "protocol id",
            overview
                .study_protocol_id
                .as_ref()
                .map_or_else(|| "not set".to_owned(), ToString::to_string),
        ),
        detail::blank(),
        detail::section("permissions"),
        detail::field("set invitation", yes_no(overview.can_set_invitation)),
        detail::field("set protocol", yes_no(overview.can_set_study_protocol)),
        detail::field("deploy", yes_no(overview.can_deploy_to_participants)),
        detail::blank(),
        detail::section("description"),
        detail::text(overview.description_line().to_owned()),
    ];

    frame.render_widget(detail::panel(theme::focused_block("Study"), lines), area);
}

/// Counts for each tab. Tabs that have not been opened say so rather than
/// showing a misleading zero.
fn render_glance(frame: &mut Frame, area: Rect, study: &StudyState) {
    let mut lines = vec![
        detail::section("contents"),
        counted_field(
            icons::participants(),
            "participants",
            study.participants.loaded,
            study.participants.total as usize,
            2,
        ),
        counted_field(
            icons::deployments(),
            "groups",
            study.details_loaded,
            study.groups().groups.len(),
            3,
        ),
        counted_field(
            icons::staff(),
            "researchers",
            study.details_loaded,
            study.researchers.len(),
            4,
        ),
        counted_field(
            icons::staff(),
            "assistants",
            study.details_loaded,
            study.assistants.len(),
            4,
        ),
        counted_field(
            icons::files(),
            "files",
            study.files_loaded,
            study.files.len(),
            5,
        ),
        counted_field(
            icons::exports(),
            "exports",
            study.exports_loaded,
            study.exports.len(),
            6,
        ),
        detail::blank(),
        detail::section("deployments"),
    ];

    if study.details_loaded {
        lines.push(detail::text(study.groups().summary()));
        for (state, count) in study.groups().state_counts() {
            lines.push(detail::bullet(
                icons::with(
                    icons::deployment_state(&state),
                    format!("{count} × {state}"),
                ),
                super::deployments::state_style(&state),
            ));
        }
    } else if study.details_loading {
        lines.push(detail::note("loading…"));
    } else {
        lines.push(detail::note("press 3 to load the deployments"));
    }

    lines.extend([
        detail::blank(),
        detail::section("getting the data"),
        detail::note(
            "Exports package study data on the server. Open the Exports tab, press n to request \
             one, and press enter to download it once its status turns available.",
        ),
    ]);

    frame.render_widget(detail::panel(theme::block("At a glance"), lines), area);
}

/// One `icon label   count` row, or a pointer to the tab that would load it.
fn counted_field(
    icon: &str,
    label: &str,
    loaded: bool,
    count: usize,
    tab: usize,
) -> ratatui::text::Line<'static> {
    let value = if loaded {
        count.to_string()
    } else {
        format!("press {tab}")
    };
    detail::field_highlighted(&icons::with(icon, label), value)
}

fn yes_no(value: bool) -> String {
    if value { "yes" } else { "no" }.to_owned()
}
