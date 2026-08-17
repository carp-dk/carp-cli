// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`].

use super::*;
use crate::app::form::{Field, FieldValue, Subject};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

fn form() -> Form {
    Form::new(
        Subject::Device("Primary Phone".to_owned()),
        vec![
            Field::new(
                "role",
                "Role name",
                FieldValue::Text("Primary Phone".to_owned()),
            )
            .with_help("How triggers refer to this device"),
            Field::new("optional", "Optional", FieldValue::Toggle(false)),
        ],
    )
}

/// Render at a size and return the whole buffer as text.
fn draw(form: &Form, width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal
        .draw(|frame| render(frame, frame.area(), form))
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
fn a_form_shows_its_fields_and_help() {
    let rendered = draw(&form(), 100, 24);

    assert!(rendered.contains("Role name"), "{rendered}");
    assert!(rendered.contains("Primary Phone"), "{rendered}");
    assert!(rendered.contains("How triggers refer"), "{rendered}");
    assert!(rendered.contains("w save"), "{rendered}");
}

/// The two modes have to look different, since they respond to different
/// keys.
#[test]
fn typing_shows_a_cursor_and_its_own_keys() {
    let mut form = form();
    form.begin_typing();
    form.clear_buffer();
    form.push('X');

    let rendered = draw(&form, 100, 24);
    assert!(rendered.contains('█'), "a cursor marks the buffer");
    assert!(rendered.contains("Esc cancel"), "{rendered}");
    assert!(!rendered.contains("w save"), "{rendered}");
}

/// A refused value has to be visible where it was typed.
#[test]
fn an_error_is_shown() {
    let mut form = form();
    form.error = Some("must be between 0 and 10".to_owned());

    let rendered = draw(&form, 100, 24);
    assert!(rendered.contains("must be between"), "{rendered}");
}

/// The overlay must render at any size the app itself allows, including
/// smaller than the form wants to be.
#[test]
fn it_renders_at_every_size() {
    let mut form = form();
    for _ in 0..30 {
        form.fields.push(Field::new(
            "extra",
            "Another field",
            FieldValue::Text("value".to_owned()),
        ));
    }

    for (width, height) in [(160, 48), (100, 24), (62, 14), (40, 8)] {
        draw(&form, width, height);
    }
}

/// A form with no fields still has to draw its chrome rather than panic.
#[test]
fn an_empty_form_renders() {
    let form = Form::new(Subject::Protocol, Vec::new());
    let rendered = draw(&form, 80, 20);
    assert!(rendered.contains("protocol"), "{rendered}");
}
