// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Building the form for a task.

use carp_protocol::Micros;
use carp_protocol::task::{KnownTask, Task};

use crate::app::form::{Field, FieldValue, Form, Subject, Vocabulary};

use super::notification_field;

/// A task, showing the participant-facing rows for the kinds that have them
/// and the class-specific rows on top.
pub fn task(task: &Task) -> Form {
    let mut fields = vec![
        Field::new("name", "Name", FieldValue::Text(task.name().to_owned()))
            .with_help("How task controls and watching triggers refer to this task"),
    ];

    if let Some(app) = task.app() {
        fields.push(
            Field::new(
                "type",
                "Card type",
                FieldValue::Catalog {
                    vocabulary: Vocabulary::AppTaskTypes,
                    value: app.r#type.clone(),
                },
            )
            .with_help("The study app picks the card's icon from this"),
        );
        fields.push(
            Field::new("title", "Title", FieldValue::Text(app.title.clone()))
                .with_help("Heading on the participant's card"),
        );
        fields.push(Field::new(
            "description",
            "Description",
            FieldValue::Text(app.description.clone()),
        ));
        fields.push(
            Field::new(
                "instructions",
                "Instructions",
                FieldValue::Text(app.instructions.clone()),
            )
            .with_help("Longer text shown once the task is opened"),
        );
    }

    if let Task::Known(known) = task {
        fields.extend(class_fields(known));
    }

    Form::new(Subject::Task(task.name().to_owned()), fields)
}

/// The rows particular to one task class.
fn class_fields(task: &KnownTask) -> Vec<Field> {
    match task {
        KnownTask::App {
            minutes_to_complete,
            notification,
            ..
        } => vec![
            minutes_field(*minutes_to_complete),
            notification_field(*notification),
        ],

        KnownTask::RpApp {
            minutes_to_complete,
            expire,
            notification,
            rp_task,
            ..
        } => vec![
            minutes_field(*minutes_to_complete),
            Field::new(
                "expire",
                "Expires after",
                FieldValue::Duration(expire.unwrap_or(Micros::ZERO)),
            )
            .with_help("How long the card stays available; 0s means it never expires"),
            notification_field(*notification),
            Field::new(
                "survey_identifier",
                "Survey identifier",
                FieldValue::Text(rp_task.identifier().to_owned()),
            )
            .with_help("Key the survey's answers are recorded under"),
            Field::new(
                "survey_navigable",
                "Branching survey",
                FieldValue::Toggle(rp_task.navigation_rules().is_some()),
            )
            .with_help("Lets steps jump elsewhere depending on the answer given"),
        ],

        KnownTask::HealthApp {
            notification,
            types,
            ..
        } => vec![
            notification_field(*notification),
            Field::new(
                "types",
                "Health metrics",
                FieldValue::CatalogSet {
                    vocabulary: Vocabulary::HealthDataTypes,
                    values: types.clone(),
                },
            )
            .with_help("Which metrics to read from the phone's health database"),
        ],

        KnownTask::Web {
            description, url, ..
        } => vec![
            Field::new(
                "web_description",
                "Description",
                FieldValue::Text(description.clone()),
            ),
            Field::new("url", "URL", FieldValue::Text(url.clone()))
                .with_help("Address the app opens when the task starts"),
        ],

        // Background and monitoring tasks are a name and their measures,
        // which are edited in the measures panel rather than here.
        KnownTask::Background { .. } | KnownTask::Monitoring { .. } => Vec::new(),
    }
}

/// The estimate shown on a task card.
///
/// Zero stands for "unstated": the wire format omits the field entirely, and
/// a spin box has no way to express absence.
fn minutes_field(minutes: Option<u32>) -> Field {
    Field::new(
        "minutes_to_complete",
        "Minutes to complete",
        FieldValue::Integer {
            value: i64::from(minutes.unwrap_or(0)),
            min: 0,
            max: 600,
        },
    )
    .with_help("Estimate shown on the card; 0 leaves it unstated")
}
