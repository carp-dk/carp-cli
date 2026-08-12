// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Building the form for a trigger.

use carp_protocol::StudyProtocol;
use carp_protocol::trigger::{DayOfWeek, KnownTrigger, Recurrence, Trigger};

use crate::app::form::{Choice, Field, FieldValue, Form, Subject, Vocabulary};

use super::device_choice;

/// A trigger, showing the schedule or condition its kind uses.
///
/// The device and watched-task rows are pickers over what the protocol
/// actually contains rather than free text, so a trigger cannot be pointed at
/// something that does not exist.
pub fn trigger(id: u32, trigger: &Trigger, protocol: &StudyProtocol) -> Form {
    let mut fields = vec![
        device_choice("source_device", "Fires on", trigger.source_device(), protocol)
            .with_help("The device this trigger is evaluated on"),
    ];

    if let Trigger::Known(known) = trigger {
        fields.extend(kind_fields(known, protocol));
    }

    Form::new(Subject::Trigger(id), fields)
}

/// The rows particular to one trigger kind.
fn kind_fields(trigger: &KnownTrigger, protocol: &StudyProtocol) -> Vec<Field> {
    match trigger {
        KnownTrigger::Periodic { period, .. } => vec![
            Field::new("period", "Every", FieldValue::Duration(*period))
                .with_help("Time between firings, e.g. 1h or 30m"),
        ],

        KnownTrigger::RecurrentScheduled {
            r#type,
            time,
            separation_count,
            day_of_week,
            ..
        } => vec![
            recurrence_field(r#type),
            Field::new("time", "At", FieldValue::Time(*time))
                .with_help("Wall-clock time in the participant's own time zone"),
            day_of_week_field(*day_of_week),
            Field::new(
                "separation_count",
                "Skip periods",
                FieldValue::Integer {
                    value: i64::from(*separation_count),
                    min: 0,
                    max: 52,
                },
            )
            .with_help("Periods to skip between firings; 0 means every period"),
        ],

        KnownTrigger::CronScheduled {
            cron_expression, ..
        } => vec![
            Field::new("cron", "Cron", FieldValue::Text(cron_expression.clone()))
                .with_help("Five fields: minute hour day month weekday"),
        ],

        KnownTrigger::UserTask {
            task_name,
            trigger_condition,
            ..
        } => vec![
            watched_task_field(task_name, protocol),
            Field::new(
                "condition",
                "When it is",
                FieldValue::Catalog {
                    vocabulary: Vocabulary::UserTaskConditions,
                    value: trigger_condition.clone(),
                },
            )
            .with_help("State the watched task has to reach, usually done"),
        ],

        KnownTrigger::NoUserTask { task_name, .. } => {
            vec![watched_task_field(task_name, protocol)]
        }

        KnownTrigger::SamplingEvent { measure_type, .. } => vec![
            Field::new(
                "measure_type",
                "Watches measure",
                FieldValue::Catalog {
                    vocabulary: Vocabulary::MeasureTypes,
                    value: measure_type.clone(),
                },
            )
            .with_help("The matching condition is kept as authored and not edited here"),
        ],

        // These three fire on the study's own lifecycle and have nothing to
        // configure.
        KnownTrigger::NoOp { .. }
        | KnownTrigger::Immediate { .. }
        | KnownTrigger::OneTime { .. } => Vec::new(),
    }
}

/// How often a recurrent trigger repeats.
///
/// Changing it also moves `period`, which the phone actually schedules on;
/// [`crate::app::form::apply`] performs both as one step.
fn recurrence_field(current: &str) -> Field {
    let options: Vec<Choice> = Recurrence::ALL
        .into_iter()
        .map(|recurrence| {
            Choice::described(
                recurrence.wire_name(),
                recurrence.wire_name(),
                format!("every {}", recurrence.period().human()),
            )
        })
        .collect();
    let selected = options
        .iter()
        .position(|choice| choice.value == current)
        .unwrap_or(0);

    Field::new(
        "recurrence",
        "Repeats",
        FieldValue::Choice { options, selected },
    )
    .with_help("Changing this also moves the period the phone schedules on")
}

fn day_of_week_field(current: Option<u8>) -> Field {
    let options: Vec<Choice> = DayOfWeek::ALL
        .into_iter()
        .map(|day| Choice::new(day.0.to_string(), day.label()))
        .collect();
    let selected = options
        .iter()
        .position(|choice| choice.value == current.unwrap_or(1).to_string())
        .unwrap_or(0);

    Field::new("day_of_week", "On", FieldValue::Choice { options, selected })
        .with_help("Only used by a weekly or biweekly schedule")
}

fn watched_task_field(current: &str, protocol: &StudyProtocol) -> Field {
    let options: Vec<Choice> = protocol
        .task_names()
        .into_iter()
        .map(|name| Choice::new(name.clone(), name))
        .collect();
    let selected = options
        .iter()
        .position(|choice| choice.value == current)
        .unwrap_or(0);

    Field::new(
        "watched_task",
        "Watches task",
        FieldValue::Choice { options, selected },
    )
    .with_help("The task whose state this trigger waits on")
}
