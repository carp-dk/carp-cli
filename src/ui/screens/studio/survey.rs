// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! The Survey tab: the pages a participant walks through.
//!
//! Steps are shown in the order the participant meets them, numbered, because
//! the order *is* the experience and a survey read out of order says nothing.

use carp_protocol::survey::{RpStep, RpTask};
use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::text::Line;
use ratatui::widgets::Row;

use crate::studio::Studio;
use crate::ui::theme;
use crate::ui::widgets::{detail, master_detail, table};

/// Draw the survey's steps and the panel describing the selected one.
pub fn render(frame: &mut Frame, area: Rect, studio: &mut Studio) {
    let Some(task_name) = studio.survey_task_name() else {
        frame.render_widget(
            table::placeholder(
                "no survey in this protocol - add an RPAppTask in the Tasks tab",
                theme::block("survey"),
            ),
            area,
        );
        return;
    };

    let Some(survey) = studio
        .protocol
        .task(&task_name)
        .and_then(carp_protocol::task::Task::survey)
        .cloned()
    else {
        return;
    };

    let (list_area, detail_area) = master_detail(area, 3, 2);
    render_steps(frame, list_area, studio, &task_name, &survey);

    let Some(detail_area) = detail_area else {
        return;
    };
    let block = theme::block("step");
    let selected = studio
        .lists
        .steps
        .selected()
        .and_then(|index| survey.steps().get(index));

    match selected {
        Some(step) => {
            frame.render_widget(detail::panel(block, step_lines(step, &survey)), detail_area)
        }
        None => frame.render_widget(detail::empty(block, "no step selected"), detail_area),
    }
}

fn render_steps(
    frame: &mut Frame,
    area: Rect,
    studio: &mut Studio,
    task_name: &str,
    survey: &RpTask,
) {
    let steps = survey.steps();
    let title = format!(
        "{task_name} {}",
        table::position_label(&studio.lists.steps, steps.len())
    );

    if steps.is_empty() {
        frame.render_widget(
            table::placeholder(
                "this survey has no steps - press a to add one",
                theme::focused_block(&title),
            ),
            area,
        );
        return;
    }

    let rows: Vec<Row> = steps
        .iter()
        .enumerate()
        .map(|(index, step)| {
            let answer = step
                .answer_format()
                .map(carp_protocol::RpAnswerFormat::label)
                .unwrap_or_else(|| step.type_label().to_owned());
            Row::new(vec![
                Line::styled(format!("{}", index + 1), theme::label()),
                Line::styled(step.identifier().to_owned(), theme::value()),
                Line::styled(answer, theme::dim()),
            ])
        })
        .collect();

    let widths = vec![
        Constraint::Length(3),
        Constraint::Fill(3),
        Constraint::Fill(3),
    ];
    let list = table::table(
        table::header(["#", "identifier", "answered with"]),
        rows,
        widths,
        theme::focused_block(&title),
    );
    table::render(frame, area, list, &mut studio.lists.steps, steps.len());
}

/// The selected step's fields, its options, and any branch out of it.
fn step_lines(step: &RpStep, survey: &RpTask) -> Vec<Line<'static>> {
    let mut lines = vec![
        detail::field("identifier", step.identifier().to_owned()),
        detail::field("type", step.type_label().to_owned()),
        detail::field("title", step.title().to_owned()),
    ];

    if let Some(format) = step.answer_format() {
        lines.push(detail::blank());
        lines.push(detail::section("answer"));
        lines.push(detail::field_highlighted("format", format.label()));
        lines.extend(choice_lines(format));
    }

    // A branch out of this step is the thing least visible in the list and
    // most consequential, so it is spelled out.
    if let Some(rules) = survey.navigation_rules()
        && let Some(rule) = rules.get(step.identifier())
    {
        lines.push(detail::blank());
        lines.push(detail::section("branches to"));
        for destination in rule.destinations() {
            let exists = survey
                .all_step_identifiers()
                .iter()
                .any(|identifier| identifier == destination);
            lines.push(detail::bullet(
                destination.to_owned(),
                if exists {
                    theme::value()
                } else {
                    theme::error()
                },
            ));
        }
    }

    if matches!(step, RpStep::Unknown(_)) {
        lines.push(detail::blank());
        lines.push(detail::note(
            "This step type is newer than this build. It is kept exactly as it \
             was, but its fields cannot be shown here.",
        ));
    }

    lines
}

/// The options of a choice question, with the value each one records.
///
/// The recorded value matters more than the label: it is what an analysis
/// sees, and a scored instrument depends on it.
fn choice_lines(format: &carp_protocol::RpAnswerFormat) -> Vec<Line<'static>> {
    use carp_protocol::survey::KnownAnswerFormat;

    let carp_protocol::RpAnswerFormat::Known(known) = format else {
        return Vec::new();
    };

    match known.as_ref() {
        KnownAnswerFormat::Choice { choices, .. } => choices
            .iter()
            .map(|choice| detail::bullet(choice.label(), theme::value()))
            .collect(),
        KnownAnswerFormat::ImageChoice { choices, .. } => choices
            .iter()
            .map(|choice| detail::bullet(choice.label(), theme::value()))
            .collect(),
        _ => Vec::new(),
    }
}
