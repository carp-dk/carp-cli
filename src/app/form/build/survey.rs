// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Building the forms for a survey step and for a measure.

use carp_protocol::Measure;
use carp_protocol::survey::{KnownStep, RpStep};

use crate::app::form::{Field, FieldValue, Form, Subject, Vocabulary};

use super::optional_field;

/// One page of a survey.
///
/// The answer format is not edited here: it has its own shape per kind - a
/// list of choices, a numeric range - and gets its own panel, reached from
/// the survey view.
pub fn survey_step(task: &str, index: usize, step: &RpStep) -> Form {
    let mut fields = vec![
        Field::new(
            "identifier",
            "Identifier",
            FieldValue::Text(step.identifier().to_owned()),
        )
        .with_help("Key this step's answer is recorded under; unique within the survey"),
        Field::new("title", "Title", FieldValue::Text(step.title().to_owned()))
            .with_help("The question or heading; often a localisation key"),
    ];

    if let RpStep::Known(known) = step {
        fields.extend(kind_fields(known));
    }

    Form::new(
        Subject::SurveyStep {
            task: task.to_owned(),
            step: index,
        },
        fields,
    )
}

/// The rows particular to one step type.
fn kind_fields(step: &KnownStep) -> Vec<Field> {
    match step {
        KnownStep::Instruction { text, optional, .. }
        | KnownStep::Completion { text, optional, .. } => vec![
            Field::new("text", "Text", FieldValue::Text(text.clone())),
            optional_field(*optional),
        ],

        KnownStep::Question {
            optional,
            auto_skip,
            timeout,
            auto_focus,
            ..
        } => vec![
            optional_field(*optional),
            Field::new("auto_skip", "Auto-advance", FieldValue::Toggle(*auto_skip))
                .with_help("Move on as soon as it is answered, without a Next tap"),
            Field::new(
                "timeout",
                "Timeout (s)",
                FieldValue::Integer {
                    value: i64::from(*timeout),
                    min: 0,
                    max: 3600,
                },
            )
            .with_help("Seconds before moving on regardless; 0 disables it"),
            Field::new(
                "auto_focus",
                "Focus on open",
                FieldValue::Toggle(*auto_focus),
            ),
        ],

        KnownStep::Form { optional, .. } => vec![optional_field(*optional)],

        KnownStep::Tapping {
            optional,
            length_of_test,
            include_instructions,
            include_results,
            ..
        } => vec![
            optional_field(*optional),
            length_field(*length_of_test),
            instructions_field(*include_instructions),
            results_field(*include_results),
        ],

        KnownStep::Flanker {
            optional,
            length_of_test,
            number_of_cards,
            include_instructions,
            include_results,
            ..
        } => vec![
            optional_field(*optional),
            length_field(*length_of_test),
            Field::new(
                "number_of_cards",
                "Cards",
                FieldValue::Integer {
                    value: i64::from(*number_of_cards),
                    min: 1,
                    max: 200,
                },
            ),
            instructions_field(*include_instructions),
            results_field(*include_results),
        ],

        KnownStep::ReactionTime {
            optional,
            length_of_test,
            switch_interval,
            include_instructions,
            include_results,
            ..
        } => vec![
            optional_field(*optional),
            length_field(*length_of_test),
            Field::new(
                "switch_interval",
                "Switch interval (s)",
                FieldValue::Integer {
                    value: i64::from(*switch_interval),
                    min: 1,
                    max: 600,
                },
            )
            .with_help("Seconds between stimuli"),
            instructions_field(*include_instructions),
            results_field(*include_results),
        ],
    }
}

fn length_field(seconds: u32) -> Field {
    Field::new(
        "length_of_test",
        "Length (s)",
        FieldValue::Integer {
            value: i64::from(seconds),
            min: 1,
            max: 3600,
        },
    )
    .with_help("How long the activity runs for")
}

fn instructions_field(on: bool) -> Field {
    Field::new(
        "include_instructions",
        "Show instructions",
        FieldValue::Toggle(on),
    )
}

fn results_field(on: bool) -> Field {
    Field::new("include_results", "Show results", FieldValue::Toggle(on))
}

/// One measure of a task.
///
/// Only the data type is editable. A measure's sampling override is a nested
/// object whose shape depends on the type, and is preserved rather than shown.
pub fn measure(task: &str, index: usize, measure: &Measure) -> Form {
    Form::new(
        Subject::Measure {
            task: task.to_owned(),
            measure: index,
        },
        vec![
            Field::new(
                "type",
                "Data type",
                FieldValue::Catalog {
                    vocabulary: Vocabulary::MeasureTypes,
                    value: measure.data_type().to_owned(),
                },
            )
            .with_help("The data stream this measure collects"),
        ],
    )
}
