// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`].

use super::*;
use crate::app::form::picker::Row as PickerRow;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

fn rows() -> Vec<PickerRow> {
    vec![
        PickerRow::new("dk.cachet.carp.survey", "survey", "used by 3 studies"),
        PickerRow::new("dk.cachet.carp.location", "location", "used by 5 studies"),
    ]
}

fn draw(picker: &mut Picker, width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal
        .draw(|frame| render(frame, frame.area(), picker))
        .unwrap();
    terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect()
}

#[test]
fn a_picker_shows_its_rows_and_their_usage() {
    let mut picker = Picker::new("measure types", PickerKind::Single, rows(), "");
    let rendered = draw(&mut picker, 100, 24);

    assert!(rendered.contains("measure types"), "{rendered}");
    assert!(rendered.contains("survey"), "{rendered}");
    assert!(rendered.contains("used by 3 studies"), "{rendered}");
    assert!(rendered.contains("Enter choose"), "{rendered}");
}

#[test]
fn the_filter_and_its_match_count_are_shown() {
    let mut picker = Picker::new("measure types", PickerKind::Single, rows(), "");
    picker.push('s');
    picker.push('u');

    let rendered = draw(&mut picker, 100, 24);
    assert!(rendered.contains("su"), "the filter text");
    assert!(rendered.contains("1 of 2"), "{rendered}");
}

/// A filter matching nothing has to say whether Enter will do anything.
#[test]
fn an_empty_result_says_whether_free_text_is_taken() {
    let mut strict = Picker::new("measure types", PickerKind::Single, rows(), "");
    for character in "zzz".chars() {
        strict.push(character);
    }
    let rendered = draw(&mut strict, 100, 24);
    assert!(rendered.contains("nothing matches"), "{rendered}");
    assert!(!rendered.contains("what you typed"), "{rendered}");

    let mut loose =
        Picker::new("measure types", PickerKind::Single, rows(), "").allowing_free_text();
    for character in "zzz".chars() {
        loose.push(character);
    }
    let rendered = draw(&mut loose, 100, 24);
    assert!(rendered.contains("what you typed"), "{rendered}");
}

/// A multi-select has to show what is already ticked.
#[test]
fn a_multi_picker_marks_its_choices() {
    let mut picker = Picker::multiple(
        "health metrics",
        rows(),
        vec!["dk.cachet.carp.survey".to_owned()],
    );

    let rendered = draw(&mut picker, 100, 24);
    assert!(rendered.contains('✓'), "{rendered}");
    assert!(rendered.contains("Space tick"), "{rendered}");
}

#[test]
fn it_renders_at_every_size() {
    let mut picker = Picker::new("measure types", PickerKind::Single, rows(), "");
    for (width, height) in [(160, 48), (100, 24), (62, 14), (30, 6)] {
        draw(&mut picker, width, height);
    }
}
