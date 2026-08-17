// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for the editing keys: adding, removing, undoing and reordering.
//!
//! Split from [`super::tests`], which covers routing and the overlay stack,
//! only so neither file grows past a screenful.

use super::tests::*;
use super::*;

/// Adding a task creates its trigger and control too, so it is startable
/// rather than orphaned.
#[test]
fn adding_a_task_wires_it_up() {
    let mut studio = studio();
    studio.section = Section::Tasks;

    press(&mut studio, KeyCode::Char('a'));
    type_text(&mut studio, "RPAppTask");
    press(&mut studio, KeyCode::Enter);
    press(&mut studio, KeyCode::Esc);

    assert_eq!(studio.protocol.tasks.len(), 1);
    assert_eq!(studio.protocol.triggers.len(), 1);
    assert_eq!(studio.protocol.task_controls.len(), 1);
    assert_eq!(errors(&studio), Vec::<String>::new());
}

/// Removing a device says what went with it, since the cascade is the whole
/// reason the operation is not a simple delete.
#[test]
fn removing_a_device_reports_the_cascade() {
    let mut studio = studio();
    studio.section = Section::Tasks;
    press(&mut studio, KeyCode::Char('a'));
    type_text(&mut studio, "BackgroundTask");
    press(&mut studio, KeyCode::Enter);
    press(&mut studio, KeyCode::Esc);

    studio.section = Section::Devices;
    let request = press(&mut studio, KeyCode::Char('x'));

    let Request::Notice(message) = request else {
        panic!("expected a notice, got {request:?}");
    };
    assert!(message.contains("and with it"), "{message}");
    assert!(message.contains("trigger"), "{message}");
}

/// Undo has to restore a cascading delete, which is the case it exists for.
#[test]
fn undo_restores_a_cascading_delete() {
    let mut studio = studio();
    studio.section = Section::Tasks;
    press(&mut studio, KeyCode::Char('a'));
    type_text(&mut studio, "BackgroundTask");
    press(&mut studio, KeyCode::Enter);
    press(&mut studio, KeyCode::Esc);

    let before = serde_json::to_value(&studio.protocol).unwrap();

    studio.section = Section::Devices;
    press(&mut studio, KeyCode::Char('x'));
    assert!(studio.protocol.primary_devices.is_empty());

    press(&mut studio, KeyCode::Char('z'));

    assert_eq!(serde_json::to_value(&studio.protocol).unwrap(), before);
    assert_eq!(errors(&studio), Vec::<String>::new());
}

#[test]
fn undo_with_nothing_to_undo_says_so() {
    let mut studio = studio();
    let request = press(&mut studio, KeyCode::Char('z'));
    assert_eq!(request, Request::Notice("nothing to undo".to_owned()));
}

/// Opening a survey moves to the Survey tab on the chosen task.
#[test]
fn a_survey_task_opens_its_survey() {
    let mut studio = studio();
    studio.section = Section::Tasks;
    press(&mut studio, KeyCode::Char('a'));
    type_text(&mut studio, "RPAppTask");
    press(&mut studio, KeyCode::Enter);
    press(&mut studio, KeyCode::Esc);

    press(&mut studio, KeyCode::Enter);

    assert_eq!(studio.section, Section::Survey);
    assert_eq!(studio.survey_task.as_deref(), Some("Survey"));
}

/// A task with no survey says so rather than opening an empty tab.
#[test]
fn a_non_survey_task_says_it_has_no_survey() {
    let mut studio = studio();
    studio.section = Section::Tasks;
    press(&mut studio, KeyCode::Char('a'));
    type_text(&mut studio, "BackgroundTask");
    press(&mut studio, KeyCode::Enter);
    press(&mut studio, KeyCode::Esc);

    let request = press(&mut studio, KeyCode::Enter);

    let Request::Notice(message) = request else {
        panic!("expected a notice, got {request:?}");
    };
    assert!(message.contains("not a survey task"), "{message}");
    assert_eq!(studio.section, Section::Tasks);
}

/// Steps are added, reordered and removed from the Survey tab.
#[test]
fn survey_steps_can_be_added_and_reordered() {
    let mut studio = studio();
    studio.section = Section::Tasks;
    press(&mut studio, KeyCode::Char('a'));
    type_text(&mut studio, "RPAppTask");
    press(&mut studio, KeyCode::Enter);
    press(&mut studio, KeyCode::Esc);
    press(&mut studio, KeyCode::Enter);

    for filter in ["Instructions", "Choice question"] {
        press(&mut studio, KeyCode::Char('a'));
        type_text(&mut studio, filter);
        press(&mut studio, KeyCode::Enter);
        press(&mut studio, KeyCode::Esc);
    }

    let steps = |studio: &Studio| {
        studio
            .protocol
            .task("Survey")
            .unwrap()
            .survey()
            .unwrap()
            .all_step_identifiers()
    };
    assert_eq!(steps(&studio).len(), 2);
    let original = steps(&studio);

    // The cursor is on the second step, which K moves up.
    press(&mut studio, KeyCode::Char('K'));
    let reordered = steps(&studio);
    assert_eq!(reordered, [original[1].clone(), original[0].clone()]);

    press(&mut studio, KeyCode::Char('x'));
    assert_eq!(steps(&studio).len(), 1);
    assert_eq!(errors(&studio), Vec::<String>::new());
}

/// The keys the editor cannot service itself have to reach the app.
#[test]
fn the_app_level_keys_are_passed_up() {
    let mut studio = studio();

    assert_eq!(press(&mut studio, KeyCode::Char('s')), Request::Save);
    assert_eq!(press(&mut studio, KeyCode::Char('o')), Request::Open);
    assert_eq!(press(&mut studio, KeyCode::Char('n')), Request::New);
    assert_eq!(press(&mut studio, KeyCode::Char('u')), Request::Upload);
    assert_eq!(press(&mut studio, KeyCode::Char('S')), Request::SyncCatalog);
    assert_eq!(press(&mut studio, KeyCode::Esc), Request::Close);
}

/// An overlay has to swallow the keys the section would otherwise act on, or
/// typing a name would start deleting things.
#[test]
fn an_overlay_swallows_the_section_keys() {
    let mut studio = studio();
    studio.section = Section::Devices;
    press(&mut studio, KeyCode::Char('a'));

    // `x` would remove a device with no picker open.
    let before = studio.protocol.devices().count();
    press(&mut studio, KeyCode::Char('x'));
    assert_eq!(studio.protocol.devices().count(), before);
    assert_eq!(studio.picker.as_ref().unwrap().filter, "x");

    // And `s` would ask the app to save.
    assert_eq!(press(&mut studio, KeyCode::Char('s')), Request::None);
}

/// A refused value keeps the form open with the reason, rather than closing
/// and losing the edit.
#[test]
fn submitting_a_refused_value_keeps_the_form_open() {
    let mut studio = studio();
    studio.section = Section::Devices;

    press(&mut studio, KeyCode::Char('e'));
    press(&mut studio, KeyCode::Enter);
    handle_key(
        &mut studio,
        KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL),
    );
    press(&mut studio, KeyCode::Enter);

    let request = press(&mut studio, KeyCode::Char('w'));

    let Request::Notice(message) = request else {
        panic!("expected a notice, got {request:?}");
    };
    assert!(message.contains("role name"), "{message}");
    assert!(studio.form.is_some(), "the form stays open to be corrected");
}

/// Every section must survive every key without panicking. The editor is
/// modal enough that an unhandled combination is easy to create.
#[test]
fn no_key_in_any_section_panics() {
    let keys = [
        KeyCode::Enter,
        KeyCode::Esc,
        KeyCode::Tab,
        KeyCode::Up,
        KeyCode::Down,
        KeyCode::Home,
        KeyCode::End,
        KeyCode::PageUp,
        KeyCode::PageDown,
        KeyCode::Backspace,
        KeyCode::Char('a'),
        KeyCode::Char('e'),
        KeyCode::Char('x'),
        KeyCode::Char('m'),
        KeyCode::Char('M'),
        KeyCode::Char('X'),
        KeyCode::Char('A'),
        KeyCode::Char('E'),
        KeyCode::Char('J'),
        KeyCode::Char('K'),
        KeyCode::Char('z'),
        KeyCode::Char('r'),
        KeyCode::Char(' '),
    ];

    for section in Section::ALL {
        // Once on an empty protocol, once on a populated one.
        for populate in [false, true] {
            let mut studio = studio();
            studio.section = section;
            if populate {
                studio.section = Section::Tasks;
                press(&mut studio, KeyCode::Char('a'));
                type_text(&mut studio, "RPAppTask");
                press(&mut studio, KeyCode::Enter);
                press(&mut studio, KeyCode::Esc);
                studio.section = section;
            }

            for code in keys {
                handle_key(&mut studio, KeyEvent::new(code, KeyModifiers::NONE));
                handle_key(&mut studio, KeyEvent::new(code, KeyModifiers::CONTROL));
            }
        }
    }
}
