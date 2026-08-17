// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! The Tasks tab: what the protocol does, and what each task collects.
//!
//! Three panes rather than two. A task's measures are a list of their own,
//! and editing them through the task's form would mean a form containing a
//! list - so the measures get their own pane under the detail panel, with
//! their own cursor.

use carp_protocol::Task;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::Row;

use crate::studio::Studio;
use crate::ui::theme;
use crate::ui::widgets::{detail, master_detail, table};

/// Draw the tasks list, the selected task's detail, and its measures.
pub fn render(frame: &mut Frame, area: Rect, studio: &mut Studio) {
    let (list_area, detail_area) = master_detail(area, 3, 3);
    render_list(frame, list_area, studio);

    let Some(detail_area) = detail_area else {
        return;
    };
    // The measures pane takes the lower third, which is enough for the
    // handful of measures a task usually has.
    let [upper, lower] =
        Layout::vertical([Constraint::Fill(2), Constraint::Fill(1)]).areas(detail_area);

    let block = theme::block("task");
    match studio.lists.selected_task(&studio.protocol) {
        Some(task) => frame.render_widget(detail::panel(block, task_lines(task, studio)), upper),
        None => frame.render_widget(detail::empty(block, "no task selected"), upper),
    }
    render_measures(frame, lower, studio);
}

fn render_list(frame: &mut Frame, area: Rect, studio: &mut Studio) {
    let tasks = &studio.protocol.tasks;
    let title = format!(
        "tasks {}",
        table::position_label(&studio.lists.tasks, tasks.len())
    );

    if tasks.is_empty() {
        frame.render_widget(
            table::placeholder("no tasks yet - press a to add one", theme::block(&title)),
            area,
        );
        return;
    }

    let rows: Vec<Row> = tasks
        .iter()
        .map(|task| {
            // Whether a trigger starts the task is the thing most worth
            // seeing at a glance: a task nothing starts never runs.
            let starts = studio.protocol.controls_for_task(task.name()).count();
            let (schedule, style) = if starts == 0 {
                ("not started".to_owned(), theme::error())
            } else {
                (
                    format!("{starts} trigger{}", if starts == 1 { "" } else { "s" }),
                    theme::dim(),
                )
            };

            Row::new(vec![
                Line::styled(task.name().to_owned(), theme::value()),
                Line::styled(task.type_label().to_owned(), theme::label()),
                Line::styled(task.measures().len().to_string(), theme::dim()),
                Line::styled(schedule, style),
            ])
        })
        .collect();

    let widths = vec![
        Constraint::Fill(3),
        Constraint::Fill(2),
        Constraint::Length(3),
        Constraint::Length(12),
    ];
    let list = table::table(
        table::header(["name", "type", "m", "started by"]),
        rows,
        widths,
        theme::focused_block(&title),
    );
    table::render(frame, area, list, &mut studio.lists.tasks, tasks.len());
}

/// The selected task's fields, and what starts it.
fn task_lines(task: &Task, studio: &Studio) -> Vec<Line<'static>> {
    let mut lines = vec![
        detail::field("name", task.name().to_owned()),
        detail::field("type", task.type_label().to_owned()),
    ];

    if let Some(app) = task.app() {
        lines.push(detail::blank());
        lines.push(detail::section("shown to the participant"));
        lines.push(detail::field("card type", app.r#type.clone()));
        lines.push(detail::field("title", app.title.clone()));
        lines.push(detail::field("description", app.description.clone()));
        if !app.instructions.is_empty() {
            lines.push(detail::field("instructions", app.instructions.clone()));
        }
    } else {
        lines.push(detail::note("runs in the background, with no card"));
    }

    if let Some(survey) = task.survey() {
        lines.push(detail::blank());
        lines.push(detail::section("survey"));
        lines.push(detail::field("identifier", survey.identifier().to_owned()));
        lines.push(detail::field("kind", survey.type_label().to_owned()));
        lines.push(detail::field_highlighted(
            "steps",
            survey.all_step_identifiers().len().to_string(),
        ));
        lines.push(detail::note("  Enter opens it in the Survey tab"));
    }

    lines.push(detail::blank());
    lines.push(detail::section("started by"));
    let controls: Vec<_> = studio.protocol.controls_for_task(task.name()).collect();
    if controls.is_empty() {
        lines.push(Line::styled(
            "  nothing starts this task, so it never runs".to_owned(),
            theme::error(),
        ));
    } else {
        for control in controls {
            let schedule = studio
                .protocol
                .triggers
                .get(&control.trigger_id)
                .map(carp_protocol::Trigger::schedule_label)
                .unwrap_or_else(|| "a missing trigger".to_owned());
            lines.push(detail::bullet(
                format!("{schedule}, on {}", control.destination_device_role_name),
                theme::value(),
            ));
        }
    }

    if matches!(task, Task::Unknown(_)) {
        lines.push(detail::blank());
        lines.push(detail::note(
            "This task class is newer than this build. It is kept exactly as it \
             was, but its fields cannot be shown here.",
        ));
    }

    lines
}

/// The selected task's measures, as their own small list.
fn render_measures(frame: &mut Frame, area: Rect, studio: &mut Studio) {
    let measures: Vec<_> = studio
        .lists
        .selected_task(&studio.protocol)
        .map(|task| task.measures().to_vec())
        .unwrap_or_default();

    let title = format!(
        "measures {} · m add · M edit · X remove",
        table::position_label(&studio.lists.measures, measures.len())
    );

    if measures.is_empty() {
        frame.render_widget(
            table::placeholder(
                "collects nothing - press m to add a measure",
                theme::block(&title),
            ),
            area,
        );
        return;
    }

    let rows: Vec<Row> = measures
        .iter()
        .map(|measure| {
            let sampling = measure
                .sampling()
                .map(carp_protocol::SamplingConfiguration::label)
                .unwrap_or_else(|| "device default".to_owned());
            Row::new(vec![
                Line::styled(measure.short_name().to_owned(), theme::value()),
                Line::styled(sampling, theme::dim()),
            ])
        })
        .collect();

    let widths = vec![Constraint::Fill(2), Constraint::Fill(3)];
    let list = table::table(
        table::header(["measure", "sampling"]),
        rows,
        widths,
        theme::block(&title),
    );
    table::render(
        frame,
        area,
        list,
        &mut studio.lists.measures,
        measures.len(),
    );
}
