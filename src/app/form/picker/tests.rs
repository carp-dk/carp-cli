// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`], the value picker overlay.

use super::*;

fn rows() -> Vec<Row> {
    vec![
        Row::new("dk.cachet.carp.survey", "survey", "used by 3 studies"),
        Row::new("dk.cachet.carp.location", "location", "used by 5 studies"),
        Row::new("dk.cachet.carp.stepcount", "stepcount", "used by demo"),
    ]
}

fn picker() -> Picker {
    Picker::new(
        "measure types",
        PickerKind::Single,
        rows(),
        "dk.cachet.carp.location",
    )
}

/// The picker opens on whatever the field already holds, so confirming
/// straight away is a no-op rather than a change.
#[test]
fn a_picker_opens_on_the_current_value() {
    let picker = picker();
    assert_eq!(picker.resolve().unwrap(), "dk.cachet.carp.location");
}

/// A value the field does not hold yet leaves the cursor at the top.
#[test]
fn an_unknown_current_value_starts_at_the_first_row() {
    let picker = Picker::new("measure types", PickerKind::Single, rows(), "nothing");
    assert_eq!(picker.selected, 0);
    assert_eq!(picker.resolve().unwrap(), "dk.cachet.carp.survey");
}

/// Filtering is why the picker exists: the measure list shares a long prefix.
#[test]
fn filtering_narrows_by_value_and_by_label() {
    let mut picker = picker();

    for character in "step".chars() {
        picker.push(character);
    }
    assert_eq!(picker.visible.len(), 1);
    assert_eq!(picker.resolve().unwrap(), "dk.cachet.carp.stepcount");

    picker.clear_filter();
    assert_eq!(picker.visible.len(), 3);

    // The namespace is matchable too, for narrowing to a package.
    for character in "cachet".chars() {
        picker.push(character);
    }
    assert_eq!(picker.visible.len(), 3);
}

/// A filter matching nothing must leave the picker in a state that resolves
/// to nothing, rather than to a stale row.
#[test]
fn a_filter_matching_nothing_resolves_to_nothing() {
    let mut picker = picker();
    for character in "zzz".chars() {
        picker.push(character);
    }

    assert!(picker.visible.is_empty());
    assert_eq!(picker.selected_row(), None);
    assert_eq!(picker.resolve(), None);
}

/// Some study has to be the first to use a new measure type, so a picker
/// that allows it takes whatever was typed.
#[test]
fn free_text_is_accepted_when_the_picker_allows_it() {
    let mut picker = picker().allowing_free_text();
    for character in "dk.cachet.carp.newthing".chars() {
        picker.push(character);
    }

    assert!(picker.visible.is_empty());
    assert_eq!(picker.resolve().unwrap(), "dk.cachet.carp.newthing");
}

/// Backspacing back to a matching filter has to bring the rows back.
#[test]
fn deleting_the_filter_restores_the_rows() {
    let mut picker = picker();
    for character in "surveyx".chars() {
        picker.push(character);
    }
    assert!(picker.visible.is_empty());

    picker.backspace();
    assert_eq!(picker.visible.len(), 1);
}

#[test]
fn the_cursor_stops_at_the_ends() {
    let mut picker = picker();

    picker.move_selection(-10);
    assert_eq!(picker.selected, 0);

    picker.move_selection(10);
    assert_eq!(picker.selected, 2);
}

/// A cursor past the end of a narrowed list would resolve to nothing or
/// panic, so filtering has to pull it back.
#[test]
fn filtering_pulls_the_cursor_back_into_range() {
    let mut picker = picker();
    picker.move_selection(10);
    assert_eq!(picker.selected, 2);

    for character in "survey".chars() {
        picker.push(character);
    }
    assert_eq!(picker.selected, 0);
    assert!(picker.selected_row().is_some());
}

/// A multi-select picker accumulates rather than replacing.
#[test]
fn a_multi_picker_toggles_values() {
    let mut picker = Picker::multiple("health metrics", rows(), vec![]);

    picker.toggle_selected();
    assert_eq!(picker.chosen, ["dk.cachet.carp.survey"]);
    assert!(picker.is_chosen("dk.cachet.carp.survey"));

    picker.move_selection(1);
    picker.toggle_selected();
    assert_eq!(picker.chosen.len(), 2);

    // Toggling again removes it.
    picker.toggle_selected();
    assert_eq!(picker.chosen, ["dk.cachet.carp.survey"]);
}

/// The chosen set has to survive filtering: narrowing the list must not
/// forget what was already ticked.
#[test]
fn choices_survive_a_filter_change() {
    let mut picker = Picker::multiple("health metrics", rows(), vec![]);
    picker.toggle_selected();

    for character in "location".chars() {
        picker.push(character);
    }
    picker.toggle_selected();
    picker.clear_filter();

    assert_eq!(picker.chosen.len(), 2);
}

/// An empty picker must not panic on any operation, since a catalogue that
/// has never been synced produces one.
#[test]
fn an_empty_picker_is_harmless() {
    let mut picker = Picker::new("measure types", PickerKind::Single, Vec::new(), "");

    picker.move_selection(1);
    picker.toggle_selected();
    picker.backspace();
    assert_eq!(picker.selected_row(), None);
    assert_eq!(picker.resolve(), None);
}

/// Catalogue rows carry the usage line, which is how someone copying a
/// convention sees who else uses it.
#[test]
fn catalogue_rows_show_how_widely_a_value_is_used() {
    let entries = vec![carp_catalog::CatalogEntry {
        value: "dk.cachet.carp.survey".to_owned(),
        occurrences: 12,
        studies: vec!["demo".to_owned(), "catch".to_owned()],
    }];

    let rows = Picker::rows_from_catalog(&entries);
    assert_eq!(rows[0].value, "dk.cachet.carp.survey");
    assert_eq!(rows[0].label, "survey");
    assert_eq!(rows[0].detail, "used by 2 studies");
}
