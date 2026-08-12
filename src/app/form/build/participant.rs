// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Building the forms for participant roles and the data expected of them.

use carp_protocol::StudyProtocol;
use carp_protocol::participant::{ExpectedParticipantData, ParticipantRole};

use crate::app::form::{Choice, Field, FieldValue, Form, Subject, Vocabulary};

/// A participant role.
pub fn participant_role(role: &ParticipantRole) -> Form {
    Form::new(
        Subject::ParticipantRole(role.role.clone()),
        vec![
            // Typed rather than picked, for the same reason a device's role
            // name is: it is a name someone chooses.
            Field::new("role", "Role", FieldValue::Text(role.role.clone()))
                .with_help("The part someone plays, e.g. Participant or Father"),
            Field::new(
                "is_optional",
                "Optional",
                FieldValue::Toggle(role.is_optional),
            )
            .with_help("An optional role need not be filled for the study to deploy"),
        ],
    )
}

/// One entry of the expected participant data.
///
/// "Asked of" offers everyone plus each defined role. Assigning to several
/// named roles at once is possible in the format but has no upstream
/// precedent, so the form offers one role or everyone; an entry that already
/// names several keeps them until it is changed.
pub fn expected_data(
    index: usize,
    expected: &ExpectedParticipantData,
    protocol: &StudyProtocol,
) -> Form {
    let mut options = vec![Choice::described(
        "",
        "everyone",
        "Whatever role they play",
    )];
    options.extend(
        protocol
            .participant_roles
            .iter()
            .map(|role| Choice::new(role.role.clone(), role.role.clone())),
    );

    let current = expected
        .assigned_to
        .role_names()
        .and_then(<[String]>::first)
        .cloned()
        .unwrap_or_default();
    let selected = options
        .iter()
        .position(|choice| choice.value == current)
        .unwrap_or(0);

    Form::new(
        Subject::ExpectedData(index),
        vec![
            Field::new(
                "input_data_type",
                "Asks for",
                FieldValue::Catalog {
                    vocabulary: Vocabulary::InputDataTypes,
                    value: expected.input_data_type().to_owned(),
                },
            )
            .with_help("What the participant is asked at enrolment"),
            Field::new(
                "assigned_to",
                "Asked of",
                FieldValue::Choice { options, selected },
            ),
        ],
    )
}
