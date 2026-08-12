// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`], the form state machine.

use super::*;
use carp_protocol::Micros;

fn form() -> Form {
    Form::new(
        Subject::Device("Primary Phone".to_owned()),
        vec![
            Field::new("role", "Role name", FieldValue::Text("Primary Phone".to_owned())),
            Field::new("optional", "Optional", FieldValue::Toggle(false)),
            Field::new("interval", "Interval", FieldValue::Duration(Micros::from_seconds(60))),
            Field::new(
                "accuracy",
                "Accuracy",
                FieldValue::Choice {
                    options: vec![Choice::new("low", "Low"), Choice::new("high", "High")],
                    selected: 0,
                },
            ),
        ],
    )
}

#[test]
fn a_new_form_browses_from_the_first_field() {
    let form = form();
    assert_eq!(form.selected, 0);
    assert!(!form.is_typing());
    assert!(!form.dirty);
}

#[test]
fn the_cursor_stops_at_the_ends() {
    let mut form = form();

    form.move_selection(-1);
    assert_eq!(form.selected, 0);

    form.move_selection(isize::MAX / 2);
    assert_eq!(form.selected, 3);

    form.move_selection(1);
    assert_eq!(form.selected, 3);
}

/// An arrow key while typing belongs to the text, not to the form: moving
/// away would silently discard the buffer.
#[test]
fn the_cursor_does_not_move_while_typing() {
    let mut form = form();
    assert!(form.begin_typing());

    form.move_selection(1);
    assert_eq!(form.selected, 0);
    assert!(form.is_typing());
}

#[test]
fn typing_replaces_a_text_field() {
    let mut form = form();
    assert!(form.begin_typing());

    form.clear_buffer();
    for character in "Study Phone".chars() {
        form.push(character);
    }
    assert!(form.commit());

    assert_eq!(form.text("role"), "Study Phone");
    assert!(form.dirty);
    assert!(!form.is_typing());
}

/// A refused value must leave the form typing so the text can be corrected,
/// with the reason visible.
#[test]
fn a_refused_value_keeps_the_form_typing() {
    let mut form = form();
    form.move_selection(2);
    assert!(form.begin_typing());

    form.clear_buffer();
    for character in "whenever".chars() {
        form.push(character);
    }

    assert!(!form.commit());
    assert!(form.is_typing(), "the text stays open for correction");
    assert!(form.error.as_ref().unwrap().contains("not a duration"));
    assert_eq!(form.duration("interval"), Some(Micros::from_seconds(60)));
}

/// Escape has to mean "leave this as it was", or nobody can trust it.
#[test]
fn cancelling_discards_the_buffer() {
    let mut form = form();
    assert!(form.begin_typing());
    form.clear_buffer();
    form.push('x');
    form.cancel_typing();

    assert_eq!(form.text("role"), "Primary Phone");
    assert!(!form.is_typing());
}

/// Toggles and choices are not typed into; space is what changes them.
#[test]
fn space_flips_a_toggle_and_steps_a_choice() {
    let mut form = form();

    form.move_selection(1);
    assert!(!form.begin_typing(), "a toggle has no text buffer");
    form.toggle_selected();
    assert!(form.flag("optional"));
    form.toggle_selected();
    assert!(!form.flag("optional"));

    form.move_selection(2);
    assert!(!form.begin_typing());
    assert_eq!(form.text("accuracy"), "low");
    form.toggle_selected();
    assert_eq!(form.text("accuracy"), "high");
    // Stepping past the end wraps rather than sticking.
    form.toggle_selected();
    assert_eq!(form.text("accuracy"), "low");
}

/// A picker writes its result into whichever field is selected.
#[test]
fn a_picker_result_lands_in_the_selected_field() {
    let mut form = Form::new(
        Subject::Task("Survey".to_owned()),
        vec![Field::new(
            "measure",
            "Measure",
            FieldValue::Catalog {
                vocabulary: Vocabulary::MeasureTypes,
                value: String::new(),
            },
        )],
    );

    form.set_selected("dk.cachet.carp.survey".to_owned());
    assert_eq!(form.text("measure"), "dk.cachet.carp.survey");
    assert!(form.dirty);
}

#[test]
fn a_multi_picker_result_replaces_the_whole_set() {
    let mut form = Form::new(
        Subject::Task("Health".to_owned()),
        vec![Field::new(
            "types",
            "Metrics",
            FieldValue::CatalogSet {
                vocabulary: Vocabulary::HealthDataTypes,
                values: vec!["STEPS".to_owned()],
            },
        )],
    );

    form.set_selected_many(vec!["WEIGHT".to_owned(), "HEIGHT".to_owned()]);
    assert_eq!(form.set("types"), ["WEIGHT", "HEIGHT"]);
}

/// Reading a value that is not there must not panic or invent one.
#[test]
fn absent_keys_read_as_empty() {
    let form = form();
    assert_eq!(form.text("nonexistent"), "");
    assert!(!form.flag("nonexistent"));
    assert_eq!(form.integer("nonexistent"), None);
    assert_eq!(form.set("nonexistent"), Vec::<String>::new());
}

/// A form with no fields must survive every operation, since a value with
/// nothing to configure produces one.
#[test]
fn an_empty_form_is_harmless() {
    let mut form = Form::new(Subject::Protocol, Vec::new());

    form.move_selection(1);
    assert!(!form.begin_typing());
    form.toggle_selected();
    form.set_selected("value".to_owned());
    assert!(!form.commit());
    assert_eq!(form.selected_field(), None);
}
