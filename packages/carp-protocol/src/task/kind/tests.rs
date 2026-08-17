// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`].

use super::*;

#[test]
fn every_kind_serialises_as_its_own_type_name() {
    for kind in TaskKind::ALL {
        let task = kind.instantiate("Example".to_owned());
        let json = serde_json::to_value(&task).unwrap();
        assert_eq!(
            json["__type"].as_str(),
            Some(kind.type_name()),
            "{} serialised as {}",
            kind.label(),
            json["__type"]
        );
        assert_eq!(TaskKind::from_type_name(kind.type_name()), Some(kind));
    }
}

#[test]
fn a_created_task_reads_back_unchanged() {
    for kind in TaskKind::ALL {
        let task = kind.instantiate("Example".to_owned());
        let json = serde_json::to_string(&task).unwrap();
        let parsed: Task = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed, task, "{} did not round trip", kind.label());
        assert_eq!(parsed.name(), "Example");
        assert_eq!(parsed.kind(), Some(kind));
    }
}

/// A monitoring task is pointless without its measures, so it is created
/// with them rather than leaving the user to remember three type strings.
#[test]
fn a_monitoring_task_ships_with_its_measures() {
    let task = TaskKind::Monitoring.instantiate("Monitoring Phone".to_owned());
    let types: Vec<&str> = task.measures().iter().map(|m| m.data_type()).collect();
    assert_eq!(types, MONITORING_MEASURES);
}

#[test]
fn survey_identifiers_are_slugs() {
    assert_eq!(survey_identifier("Sleep Diary"), "sleep_diary");
    assert_eq!(survey_identifier("WHO-5  Wellbeing!"), "who_5_wellbeing");
    assert_eq!(survey_identifier("  "), "survey");
    assert_eq!(survey_identifier("Ω survey"), "ω_survey");
}
