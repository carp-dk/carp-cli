// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`].

use super::*;

#[test]
fn a_role_round_trips() {
    let original = serde_json::json!({ "role": "Participant", "isOptional": false });
    let role: ParticipantRole = serde_json::from_value(original.clone()).unwrap();
    assert_eq!(serde_json::to_value(&role).unwrap(), original);
}

#[test]
fn expected_data_for_a_role_round_trips() {
    let original = serde_json::json!({
        "attribute": {
            "__type": "dk.cachet.carp.common.application.users.ParticipantAttribute.DefaultParticipantAttribute",
            "inputDataType": "dk.carp.webservices.input.informed_consent"
        },
        "assignedTo": {
            "__type": "dk.cachet.carp.common.application.users.AssignedTo.Roles",
            "roleNames": ["Participant"]
        }
    });

    let expected: ExpectedParticipantData = serde_json::from_value(original.clone()).unwrap();
    assert_eq!(
        expected.input_data_type(),
        "dk.carp.webservices.input.informed_consent"
    );
    assert_eq!(expected.assigned_to.label(), "Participant");
    assert_eq!(serde_json::to_value(&expected).unwrap(), original);
}

/// `AssignedTo.All` is a Kotlin object: an empty JSON body with only a
/// discriminator. It must not gain fields or lose its braces.
#[test]
fn assigned_to_everyone_is_an_empty_object() {
    let expected = ExpectedParticipantData::for_everyone("dk.cachet.carp.input.sex");
    let json = serde_json::to_value(&expected).unwrap();

    assert_eq!(
        json["assignedTo"],
        serde_json::json!({
            "__type": "dk.cachet.carp.common.application.users.AssignedTo.All"
        })
    );

    let parsed: ExpectedParticipantData = serde_json::from_value(json).unwrap();
    assert_eq!(parsed, expected);
}

/// Renaming a role has to reach the expected-data entries naming it, or
/// the study asks a role that no longer exists for its consent.
#[test]
fn renaming_a_role_reaches_its_assignments() {
    let mut expected =
        ExpectedParticipantData::for_roles("dk.cachet.carp.input.sex", vec!["Old".to_owned()]);

    expected.assigned_to.rename_role("Old", "New");
    assert_eq!(expected.assigned_to.role_names().unwrap(), ["New"]);

    // Renaming something else leaves it alone.
    expected.assigned_to.rename_role("Absent", "Wrong");
    assert_eq!(expected.assigned_to.role_names().unwrap(), ["New"]);
}
