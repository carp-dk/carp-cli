// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! The Triggers tab: when each task starts.

use carp_protocol::Trigger;
use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::text::Line;
use ratatui::widgets::Row;

use crate::studio::Studio;
use crate::ui::theme;
use crate::ui::widgets::{detail, master_detail, table};

/// Draw the triggers list and the panel describing the selected one.
pub fn render(frame: &mut Frame, area: Rect, studio: &mut Studio) {
    let (list_area, detail_area) = master_detail(area, 3, 2);
    render_list(frame, list_area, studio);

    let Some(detail_area) = detail_area else {
        return;
    };
    let block = theme::block("trigger");
    match studio.lists.selected_trigger(&studio.protocol) {
        Some((id, trigger)) => frame.render_widget(
            detail::panel(block, trigger_lines(id, trigger, studio)),
            detail_area,
        ),
        None => frame.render_widget(detail::empty(block, "no trigger selected"), detail_area),
    }
}

fn render_list(frame: &mut Frame, area: Rect, studio: &mut Studio) {
    let triggers = &studio.protocol.triggers;
    let title = format!(
        "triggers {}",
        table::position_label(&studio.lists.triggers, triggers.len())
    );

    if triggers.is_empty() {
        frame.render_widget(
            table::placeholder("no triggers yet - press a to add one", theme::block(&title)),
            area,
        );
        return;
    }

    let rows: Vec<Row> = triggers
        .iter()
        .map(|(id, trigger)| {
            // A trigger that starts nothing is inert, which is worth seeing
            // in the list rather than only in the checks.
            let starts = studio.protocol.controls_for_trigger(*id).count();
            let (tasks, style) = if starts == 0 {
                ("starts nothing".to_owned(), theme::warn())
            } else {
                (
                    studio
                        .protocol
                        .controls_for_trigger(*id)
                        .map(|control| control.task_name.as_str())
                        .collect::<Vec<_>>()
                        .join(", "),
                    theme::value(),
                )
            };

            Row::new(vec![
                Line::styled(id.to_string(), theme::label()),
                Line::styled(trigger.schedule_label(), theme::value()),
                Line::styled(tasks, style),
            ])
        })
        .collect();

    let widths = vec![
        Constraint::Length(3),
        Constraint::Fill(3),
        Constraint::Fill(3),
    ];
    let list = table::table(
        table::header(["id", "fires", "starts"]),
        rows,
        widths,
        theme::focused_block(&title),
    );
    table::render(frame, area, list, &mut studio.lists.triggers, triggers.len());
}

/// The selected trigger's schedule, source and effects.
fn trigger_lines(id: u32, trigger: &Trigger, studio: &Studio) -> Vec<Line<'static>> {
    let source_exists = studio.protocol.device(trigger.source_device()).is_some();

    let mut lines = vec![
        detail::field("id", id.to_string()),
        detail::field("type", trigger.type_label().to_owned()),
        detail::field_highlighted("fires", trigger.schedule_label()),
        detail::field_styled(
            "on device",
            trigger.source_device().to_owned(),
            if source_exists {
                theme::value()
            } else {
                theme::error()
            },
        ),
    ];

    if !source_exists {
        lines.push(detail::note("  that device is not in this protocol"));
    }

    if let Some(watched) = trigger.watched_task() {
        let exists = studio.protocol.task(watched).is_some();
        lines.push(detail::field_styled(
            "watches",
            watched.to_owned(),
            if exists { theme::value() } else { theme::error() },
        ));
        if !exists {
            lines.push(detail::note("  that task is not in this protocol"));
        }
    }

    lines.push(detail::blank());
    lines.push(detail::section("starts"));
    let controls: Vec<_> = studio.protocol.controls_for_trigger(id).collect();
    if controls.is_empty() {
        lines.push(detail::note("  nothing - this trigger has no effect"));
    } else {
        for control in controls {
            lines.push(detail::bullet(
                format!(
                    "{} on {}",
                    control.task_name, control.destination_device_role_name
                ),
                theme::value(),
            ));
        }
    }

    if matches!(trigger, Trigger::Unknown(_)) {
        lines.push(detail::blank());
        lines.push(detail::note(
            "This trigger class is newer than this build. It is kept exactly as \
             it was, but its settings cannot be shown here.",
        ));
    }

    lines
}
