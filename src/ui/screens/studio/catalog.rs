// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! The Catalog tab: which version of CARP's conventions the editor is
//! offering, and the studies it learned them from.
//!
//! This pane is the answer to "where do the options in those pickers come
//! from?". It names the upstream commit, says how many studies contributed,
//! and lists them as templates to start from.

use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::text::Line;
use ratatui::widgets::Row;

use crate::studio::{CatalogState, Studio};
use crate::ui::theme;
use crate::ui::widgets::{detail, master_detail, table};

/// Draw the templates list and the panel describing the catalogue.
pub fn render(frame: &mut Frame, area: Rect, studio: &mut Studio) {
    let (list_area, detail_area) = master_detail(area, 3, 2);
    render_templates(frame, list_area, studio);

    if let Some(detail_area) = detail_area {
        frame.render_widget(
            detail::panel(theme::block("catalogue"), catalog_lines(studio)),
            detail_area,
        );
    }
}

fn render_templates(frame: &mut Frame, area: Rect, studio: &mut Studio) {
    let templates = &studio.catalog.templates;
    let title = format!(
        "studies {}",
        table::position_label(&studio.lists.templates, templates.len())
    );

    if templates.is_empty() {
        let message = match &studio.catalog_state {
            CatalogState::Syncing => "downloading the upstream studies…",
            CatalogState::Failed(_) => "the catalogue could not be downloaded - press S to retry",
            _ => "no catalogue yet - press S to download the upstream studies",
        };
        frame.render_widget(
            table::placeholder(message, theme::focused_block(&title)),
            area,
        );
        return;
    }

    let rows: Vec<Row> = templates
        .iter()
        .map(|template| {
            Row::new(vec![
                Line::styled(template.study.clone(), theme::value()),
                Line::styled(template.name.clone(), theme::label()),
                Line::styled(template.summary.clone(), theme::dim()),
            ])
        })
        .collect();

    let widths = vec![
        Constraint::Fill(1),
        Constraint::Fill(2),
        Constraint::Fill(2),
    ];
    let list = table::table(
        table::header(["study", "protocol", "contains"]),
        rows,
        widths,
        theme::focused_block(&title),
    );
    table::render(
        frame,
        area,
        list,
        &mut studio.lists.templates,
        templates.len(),
    );
}

/// The catalogue's version, size, and whether upstream has moved on.
fn catalog_lines(studio: &Studio) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    match &studio.catalog_state {
        CatalogState::Absent => {
            lines.push(detail::note("No catalogue has been downloaded yet."));
            lines.push(detail::blank());
            lines.push(detail::text(
                "The editor's measure types, health metrics and other options \
                 are learned from the studies in carp_study_app_configurations \
                 rather than fixed in this build. Press S to download them.",
            ));
            return lines;
        }
        CatalogState::Syncing => {
            lines.push(Line::styled("downloading…".to_owned(), theme::warn()));
            lines.push(detail::blank());
        }
        CatalogState::Failed(error) => {
            lines.push(Line::styled("download failed".to_owned(), theme::error()));
            lines.push(detail::text(error.clone()));
            lines.push(detail::blank());
        }
        CatalogState::Ready => {}
    }

    if let Some(version) = studio.catalog.version.as_ref() {
        lines.push(detail::section("derived from"));
        lines.push(detail::field("repository", version.repository.clone()));
        lines.push(detail::field_highlighted(
            "commit",
            version.commit.short_sha().to_owned(),
        ));
        lines.push(detail::field("subject", version.commit.subject.clone()));
        lines.push(detail::field("dated", version.commit.date.clone()));
        lines.push(detail::field(
            "downloaded",
            match version.age_in_days() {
                Some(0) => "today".to_owned(),
                Some(1) => "yesterday".to_owned(),
                Some(days) => format!("{days} days ago"),
                None => version.fetched_at.clone(),
            },
        ));
        lines.push(detail::field("studies", version.studies.to_string()));
        lines.push(detail::blank());
    }

    // An available update is stated, never applied: replacing the vocabulary
    // mid-edit would make a value vanish from a list someone is looking at.
    match studio.update_available.as_ref() {
        Some(commit) => {
            lines.push(Line::styled(
                format!("upstream has moved to {}", commit.short_sha()),
                theme::warn(),
            ));
            lines.push(detail::note(format!("  {}", commit.subject)));
            lines.push(detail::note("  press S to update"));
        }
        None if studio.catalog_state == CatalogState::Ready => {
            lines.push(Line::styled("up to date".to_owned(), theme::ok()));
        }
        None => {}
    }

    lines.push(detail::blank());
    lines.push(detail::section("vocabulary"));
    for (label, entries) in [
        ("measure types", studio.catalog.measure_types.len()),
        ("device classes", studio.catalog.device_types.len()),
        ("health metrics", studio.catalog.health_data_types.len()),
        ("input types", studio.catalog.input_data_types.len()),
        ("task types", studio.catalog.app_task_types.len()),
        ("question types", studio.catalog.question_types.len()),
        ("device names", studio.catalog.device_role_names.len()),
        ("participant roles", studio.catalog.participant_roles.len()),
    ] {
        lines.push(detail::field(label, entries.to_string()));
    }

    if !studio.catalog.skipped.is_empty() {
        lines.push(detail::blank());
        lines.push(Line::styled(
            format!("{} study could not be read", studio.catalog.skipped.len()),
            theme::warn(),
        ));
        for skipped in &studio.catalog.skipped {
            lines.push(detail::note(format!("  {skipped}")));
        }
    }

    lines.push(detail::blank());
    lines.push(detail::note(
        "Enter starts a new protocol from the selected study",
    ));
    lines
}
