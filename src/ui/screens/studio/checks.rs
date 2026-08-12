// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! The Checks tab: what is wrong with the protocol, and what to do about it.
//!
//! Findings are listed worst first. The panel beside them carries the hint,
//! because a finding without a remedy is only half useful - "trigger 3 fires
//! on a device that is not in this protocol" is a fact, "point it at one of
//! the protocol's devices" is what to do.

use carp_protocol::validate::{Diagnostic, Severity};
use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::Row;

use crate::studio::Studio;
use crate::ui::theme;
use crate::ui::widgets::{detail, master_detail, table};

/// Draw the findings and the panel describing the selected one.
pub fn render(frame: &mut Frame, area: Rect, studio: &mut Studio) {
    let (list_area, detail_area) = master_detail(area, 3, 2);
    render_findings(frame, list_area, studio);

    let Some(detail_area) = detail_area else {
        return;
    };
    let block = theme::block("finding");
    let selected = studio
        .lists
        .checks
        .selected()
        .and_then(|index| studio.diagnostics.get(index));

    match selected {
        Some(diagnostic) => {
            frame.render_widget(detail::panel(block, finding_lines(diagnostic)), detail_area);
        }
        None => frame.render_widget(
            detail::empty(block, "nothing to report"),
            detail_area,
        ),
    }
}

fn render_findings(frame: &mut Frame, area: Rect, studio: &mut Studio) {
    let diagnostics = &studio.diagnostics;
    let title = format!(
        "checks {}",
        table::position_label(&studio.lists.checks, diagnostics.len())
    );

    if diagnostics.is_empty() {
        frame.render_widget(
            table::placeholder(
                "nothing to report - this protocol is ready to upload",
                theme::focused_block(&title),
            ),
            area,
        );
        return;
    }

    let rows: Vec<Row> = diagnostics
        .iter()
        .map(|diagnostic| {
            Row::new(vec![
                Line::styled(diagnostic.severity.label(), severity_style(diagnostic.severity)),
                Line::styled(diagnostic.location.clone(), theme::label()),
                Line::styled(diagnostic.message.clone(), theme::value()),
            ])
        })
        .collect();

    let widths = vec![
        Constraint::Length(7),
        Constraint::Fill(2),
        Constraint::Fill(5),
    ];
    let list = table::table(
        table::header(["", "where", "what"]),
        rows,
        widths,
        theme::focused_block(&title),
    );
    table::render(
        frame,
        area,
        list,
        &mut studio.lists.checks,
        diagnostics.len(),
    );
}

/// One finding, spelled out with its remedy.
fn finding_lines(diagnostic: &Diagnostic) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::styled(
            diagnostic.severity.label().to_uppercase(),
            severity_style(diagnostic.severity),
        ),
        detail::blank(),
        detail::field("where", diagnostic.location.clone()),
        detail::blank(),
        detail::text(diagnostic.message.clone()),
    ];

    if let Some(hint) = &diagnostic.hint {
        lines.push(detail::blank());
        lines.push(detail::section("what to do"));
        lines.push(detail::note(hint.clone()));
    }

    lines.push(detail::blank());
    lines.push(detail::note(match diagnostic.severity {
        Severity::Error => "An error stops the protocol being uploaded.",
        Severity::Warning => "A warning is legal, but rarely what was meant.",
        Severity::Info => "Worth knowing; nothing to fix.",
    }));

    lines
}

fn severity_style(severity: Severity) -> Style {
    match severity {
        Severity::Error => theme::error(),
        Severity::Warning => theme::warn(),
        Severity::Info => theme::dim(),
    }
}
