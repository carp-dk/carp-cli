// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`], driving the editor the way a user does: with keys.
//!
//! Rather than calling the actions directly, these press the keys and check
//! the protocol that comes out. That covers the routing as well as the
//! actions, which is where an overlay swallowing the wrong key would hide.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::*;
use carp_protocol::validate::Severity;

pub(super) fn studio() -> Studio {
    Studio::new("979b408d-784e-4b1b-bb1e-ff9204e072f3".to_owned())
}

/// Press one key.
pub(super) fn press(studio: &mut Studio, code: KeyCode) -> Request {
    handle_key(studio, KeyEvent::new(code, KeyModifiers::NONE))
}

/// Type a run of characters.
pub(super) fn type_text(studio: &mut Studio, text: &str) {
    for character in text.chars() {
        press(studio, KeyCode::Char(character));
    }
}

/// Validation errors, as readable lines.
pub(super) fn errors(studio: &Studio) -> Vec<String> {
    studio
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
        .map(|diagnostic| format!("[{}] {}", diagnostic.location, diagnostic.message))
        .collect()
}

/// A blank protocol has to be deployable-shaped from the start, or the
/// Checks tab greets a new user with an error they did not cause.
#[test]
fn a_new_protocol_starts_valid() {
    let studio = studio();
    assert_eq!(errors(&studio), Vec::<String>::new());
    assert!(!studio.dirty);
    assert_eq!(studio.protocol.primary_devices.len(), 1);
}

/// The number keys reach every tab, and Tab cycles.
#[test]
fn the_tabs_are_reachable() {
    let mut studio = studio();

    press(&mut studio, KeyCode::Char('3'));
    assert_eq!(studio.section, Section::Tasks);

    press(&mut studio, KeyCode::Tab);
    assert_eq!(studio.section, Section::Triggers);

    press(&mut studio, KeyCode::BackTab);
    assert_eq!(studio.section, Section::Tasks);
}

/// Adding a device: `a` opens the picker, Enter creates it, and the form for
/// the new device opens so it can be named straight away.
#[test]
fn adding_a_device_opens_a_picker_then_its_form() {
    let mut studio = studio();
    studio.section = Section::Devices;

    press(&mut studio, KeyCode::Char('a'));
    assert!(studio.picker.is_some(), "the picker opened");

    // Filter to the location service and take it.
    type_text(&mut studio, "LocationService");
    press(&mut studio, KeyCode::Enter);

    assert!(studio.picker.is_none());
    assert!(studio.form.is_some(), "the new device opened for editing");
    assert!(studio.protocol.device("Location Service").is_some());
    // A connected device is wired up, so it is reachable.
    assert_eq!(errors(&studio), Vec::<String>::new());
}

/// Escape closes the picker without creating anything.
#[test]
fn escaping_a_picker_creates_nothing() {
    let mut studio = studio();
    studio.section = Section::Devices;
    let before = studio.protocol.devices().count();

    press(&mut studio, KeyCode::Char('a'));
    press(&mut studio, KeyCode::Esc);

    assert!(studio.picker.is_none());
    assert!(studio.creating.is_none());
    assert_eq!(studio.protocol.devices().count(), before);
}

/// Editing a value end to end: open the form, type, commit the field, submit
/// the form.
#[test]
fn a_field_can_be_typed_and_submitted() {
    let mut studio = studio();
    studio.section = Section::Devices;

    press(&mut studio, KeyCode::Char('e'));
    assert!(studio.form.is_some());

    // Enter opens the text cursor on the role-name field.
    press(&mut studio, KeyCode::Enter);
    assert!(studio.form.as_ref().unwrap().is_typing());

    handle_key(
        &mut studio,
        KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL),
    );
    type_text(&mut studio, "Study Phone");
    press(&mut studio, KeyCode::Enter);
    assert!(!studio.form.as_ref().unwrap().is_typing());

    press(&mut studio, KeyCode::Char('w'));

    assert!(studio.form.is_none());
    assert!(studio.protocol.device("Study Phone").is_some());
    assert!(studio.dirty);
    assert_eq!(errors(&studio), Vec::<String>::new());
}

/// Escape out of a form has to discard, or nobody can trust it.
#[test]
fn escaping_a_form_discards_the_edit() {
    let mut studio = studio();
    studio.section = Section::Devices;

    press(&mut studio, KeyCode::Char('e'));
    press(&mut studio, KeyCode::Enter);
    handle_key(
        &mut studio,
        KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL),
    );
    type_text(&mut studio, "Discarded");
    press(&mut studio, KeyCode::Enter);
    press(&mut studio, KeyCode::Esc);

    assert!(studio.form.is_none());
    assert!(studio.protocol.device("Primary Phone").is_some());
    assert!(!studio.dirty);
}
