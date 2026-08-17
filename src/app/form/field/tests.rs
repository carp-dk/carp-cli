// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`], the field types the editor is built from.

use super::*;

/// A field must refuse what it cannot store, and say why. Silently keeping
/// the old value would look like the keystroke was lost.
#[test]
fn a_number_outside_its_bounds_is_refused_with_a_reason() {
    let mut value = FieldValue::Integer {
        value: 5,
        min: 0,
        max: 10,
    };

    assert!(value.accept_text("7").is_ok());
    assert_eq!(value.as_integer(), Some(7));

    let error = value.accept_text("99").unwrap_err();
    assert_eq!(error, "must be between 0 and 10");
    assert_eq!(value.as_integer(), Some(7), "the old value is kept");

    assert!(
        value
            .accept_text("four")
            .unwrap_err()
            .contains("whole number")
    );
}

/// Durations are where a wrong unit is invisible, so the field parses the
/// human form rather than taking a raw microsecond count.
#[test]
fn a_duration_is_typed_in_human_units() {
    let mut value = FieldValue::Duration(Micros::ZERO);

    assert!(value.accept_text("30d").is_ok());
    assert_eq!(value.as_duration(), Some(Micros::from_days(30)));
    assert_eq!(value.display(), "30d");

    assert!(value.accept_text("1h30m").is_ok());
    assert_eq!(value.as_duration(), Some(Micros::from_minutes(90)));

    assert!(
        value
            .accept_text("soon")
            .unwrap_err()
            .contains("not a duration")
    );
}

#[test]
fn a_time_is_typed_as_hours_and_minutes() {
    let mut value = FieldValue::Time(TimeOfDay::new(0, 0));

    assert!(value.accept_text("20:00").is_ok());
    assert_eq!(value.as_time(), Some(TimeOfDay::new(20, 0)));
    assert_eq!(value.display(), "20:00");

    assert!(
        value
            .accept_text("25:00")
            .unwrap_err()
            .contains("not a time")
    );
    assert_eq!(value.as_time(), Some(TimeOfDay::new(20, 0)));
}

/// The fields that are not typed into have to say so, or the form would open
/// a text cursor on a toggle.
#[test]
fn untyped_fields_have_no_editable_text() {
    let untyped = [
        FieldValue::Toggle(true),
        FieldValue::Choice {
            options: vec![Choice::new("a", "A")],
            selected: 0,
        },
        FieldValue::Catalog {
            vocabulary: Vocabulary::MeasureTypes,
            value: String::new(),
        },
        FieldValue::CatalogSet {
            vocabulary: Vocabulary::HealthDataTypes,
            values: Vec::new(),
        },
    ];

    for mut value in untyped {
        assert_eq!(value.editable_text(), None, "{value:?}");
        assert!(value.accept_text("anything").is_err(), "{value:?}");
    }
}

/// Every typed field must round-trip through its own display form, or
/// opening a field and pressing enter without changing anything would alter
/// the value.
#[test]
fn opening_and_committing_a_field_unchanged_keeps_its_value() {
    let values = [
        FieldValue::Text("Primary Phone".to_owned()),
        FieldValue::Integer {
            value: 42,
            min: 0,
            max: 100,
        },
        FieldValue::Duration(Micros::from_days(5)),
        FieldValue::Time(TimeOfDay {
            hour: 8,
            minute: 5,
            second: 30,
        }),
    ];

    for original in values {
        let mut value = original.clone();
        let text = value.editable_text().expect("a typed field");
        value
            .accept_text(&text)
            .unwrap_or_else(|error| panic!("{original:?} rejected its own text: {error}"));
        assert_eq!(value, original);
    }
}

/// An empty value reads as a dash rather than as nothing, so a blank row is
/// visibly blank rather than looking like a rendering fault.
#[test]
fn empty_values_render_as_a_dash() {
    assert_eq!(FieldValue::Text(String::new()).display(), "—");
    assert_eq!(
        FieldValue::Catalog {
            vocabulary: Vocabulary::MeasureTypes,
            value: String::new()
        }
        .display(),
        "—"
    );
    assert_eq!(
        FieldValue::CatalogSet {
            vocabulary: Vocabulary::HealthDataTypes,
            values: Vec::new()
        }
        .display(),
        "—"
    );
}

/// A set says how many it holds once there is more than one, since the
/// values themselves would not fit the row.
#[test]
fn a_set_summarises_itself() {
    let set = |values: &[&str]| FieldValue::CatalogSet {
        vocabulary: Vocabulary::HealthDataTypes,
        values: values.iter().map(|value| (*value).to_owned()).collect(),
    };

    assert_eq!(set(&["STEPS"]).display(), "STEPS");
    assert_eq!(set(&["STEPS", "WEIGHT"]).display(), "2 selected");
}

/// Every vocabulary must resolve to a list, or a picker would open empty with
/// no way to tell whether that is a fault or an empty catalogue.
#[test]
fn every_vocabulary_resolves_to_a_catalogue_list() {
    let catalog = carp_catalog::Catalog::default();
    let vocabularies = [
        Vocabulary::MeasureTypes,
        Vocabulary::HealthDataTypes,
        Vocabulary::InputDataTypes,
        Vocabulary::AppTaskTypes,
        Vocabulary::UserTaskConditions,
        Vocabulary::UploadMethods,
        Vocabulary::LocationAccuracies,
    ];

    for vocabulary in vocabularies {
        assert!(vocabulary.entries(&catalog).is_empty());
        assert!(!vocabulary.title().is_empty());
    }
}
