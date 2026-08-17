// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Turning a part of a protocol into a [`Form`].
//!
//! One function per kind of thing that can be edited, one module per family.
//! Each form lists only the fields that value actually has - a `PolarDevice`
//! has no sampling interval, so its form has no row for one - which is what
//! keeps the editor readable rather than a wall of mostly-inapplicable
//! settings.
//!
//! Field keys are the contract with [`super::apply`]: the two have to agree,
//! and the tests there submit forms built here, so a mismatch fails a test
//! rather than silently dropping an edit.

pub mod device;
pub mod endpoint;
pub mod participant;
pub mod survey;
pub mod task;
pub mod trigger;

pub use device::device;
pub use endpoint::data_end_point;
pub use participant::{expected_data, participant_role};
pub use survey::{measure, survey_step};
pub use task::task;
pub use trigger::trigger;

use carp_protocol::StudyProtocol;

use super::{Choice, Field, FieldValue, Form, Subject};

/// The protocol's own identity.
pub fn protocol(protocol: &StudyProtocol) -> Form {
    Form::new(
        Subject::Protocol,
        vec![
            Field::new("name", "Name", FieldValue::Text(protocol.name.clone()))
                .with_help("How the protocol is listed in CAWS"),
            Field::new(
                "description",
                "Description",
                FieldValue::Text(protocol.description.clone().unwrap_or_default()),
            )
            .with_help("Free text, or a key such as study.description.description"),
            Field::new(
                "owner_id",
                "Owner id",
                FieldValue::Text(protocol.owner_id.clone()),
            )
            .with_help("A UUID. CAWS replaces it with the uploading account's id"),
        ],
    )
}

/// The CAMS `applicationData` block: how the study presents itself, and where
/// its data goes.
pub fn application_data(protocol: &StudyProtocol) -> Form {
    let data = protocol.application_data.clone().unwrap_or_default();
    let description = data.study_description.unwrap_or_default();
    let responsible = description.responsible.unwrap_or_default();

    Form::new(
        Subject::ApplicationData,
        vec![
            Field::new(
                "api_level",
                "Protocol API level",
                FieldValue::Choice {
                    options: vec![
                        Choice::described("", "unset", "For study apps predating the level"),
                        Choice::described("2.0", "2.0", "CAMS 2.0 and later"),
                    ],
                    selected: usize::from(data.protocol_api_level.as_deref() == Some("2.0")),
                },
            )
            .with_help("Which study-app release this document is written for"),
            Field::new(
                "application_name",
                "Application name",
                FieldValue::Text(data.application_name.unwrap_or_default()),
            )
            .with_help("Flutter application id, e.g. neuropathy_tracker"),
            Field::new("title", "Study title", FieldValue::Text(description.title))
                .with_help("Shown to the participant; often a localisation key"),
            Field::new(
                "study_description",
                "Study description",
                FieldValue::Text(description.description),
            ),
            Field::new("purpose", "Purpose", FieldValue::Text(description.purpose)),
            Field::new(
                "responsible_name",
                "Responsible",
                FieldValue::Text(responsible.name),
            )
            .with_help("Who is accountable for the study"),
            Field::new(
                "responsible_email",
                "Responsible email",
                FieldValue::Text(responsible.email),
            ),
            Field::new(
                "responsible_affiliation",
                "Affiliation",
                FieldValue::Text(responsible.affiliation),
            ),
        ],
    )
}

/// A choice field listing the protocol's devices by role name.
///
/// Shared by the trigger and task-control forms, which both have to point at
/// a device that exists rather than accept a typed name.
pub(super) fn device_choice(
    key: &'static str,
    label: &str,
    current: &str,
    protocol: &StudyProtocol,
) -> Field {
    let options: Vec<Choice> = protocol
        .device_role_names()
        .into_iter()
        .map(|role| Choice::new(role.clone(), role))
        .collect();
    let selected = options
        .iter()
        .position(|choice| choice.value == current)
        .unwrap_or(0);

    Field::new(key, label, FieldValue::Choice { options, selected })
}

/// A toggle for whether a survey step may be skipped.
pub(super) fn optional_field(optional: bool) -> Field {
    Field::new("optional", "Skippable", FieldValue::Toggle(optional))
        .with_help("Whether the participant may move past without answering")
}

/// A toggle for whether an app task raises a phone notification.
pub(super) fn notification_field(on: bool) -> Field {
    Field::new("notification", "Notify", FieldValue::Toggle(on))
        .with_help("Raise a phone notification when the task appears")
}
