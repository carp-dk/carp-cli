// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Keys while a form or a picker is open.
//!
//! These take precedence over the section beneath them, which is what makes
//! `Esc` mean "close the thing in front of me" at every depth.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::studio::{Studio, pickers};

use super::{PAGE_STEP, Request};

/// Keys while a picker is open.
pub(super) fn picker_key(studio: &mut Studio, key: KeyEvent) -> Request {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        if let Some(picker) = studio.picker.as_mut() {
            match key.code {
                KeyCode::Char('u') => picker.clear_filter(),
                KeyCode::Char('d') => picker.move_selection(PAGE_STEP),
                _ => {}
            }
        }
        return Request::None;
    }

    match key.code {
        KeyCode::Esc => {
            studio.picker = None;
            studio.creating = None;
            Request::None
        }
        KeyCode::Enter => match pickers::resolve(studio) {
            Some(message) => Request::Notice(message),
            None => Request::None,
        },
        KeyCode::Up => {
            if let Some(picker) = studio.picker.as_mut() {
                picker.move_selection(-1);
            }
            Request::None
        }
        KeyCode::Down => {
            if let Some(picker) = studio.picker.as_mut() {
                picker.move_selection(1);
            }
            Request::None
        }
        KeyCode::PageUp => {
            if let Some(picker) = studio.picker.as_mut() {
                picker.move_selection(-PAGE_STEP);
            }
            Request::None
        }
        KeyCode::PageDown => {
            if let Some(picker) = studio.picker.as_mut() {
                picker.move_selection(PAGE_STEP);
            }
            Request::None
        }
        KeyCode::Backspace => {
            if let Some(picker) = studio.picker.as_mut() {
                picker.backspace();
            }
            Request::None
        }
        // Space ticks a row in a multi-select. Elsewhere it is an ordinary
        // character, since a filter may legitimately contain one.
        KeyCode::Char(' ') => {
            if let Some(picker) = studio.picker.as_mut() {
                if picker.kind == crate::app::form::picker::PickerKind::Multiple {
                    picker.toggle_selected();
                } else {
                    picker.push(' ');
                }
            }
            Request::None
        }
        KeyCode::Char(character) => {
            if let Some(picker) = studio.picker.as_mut() {
                picker.push(character);
            }
            Request::None
        }
        _ => Request::None,
    }
}

/// Keys while a form is open.
pub(super) fn form_key(studio: &mut Studio, key: KeyEvent) -> Request {
    let typing = studio.form.as_ref().is_some_and(crate::app::form::Form::is_typing);

    if typing {
        return typing_key(studio, key);
    }

    match key.code {
        KeyCode::Esc => {
            studio.form = None;
            Request::None
        }
        KeyCode::Up | KeyCode::Char('k') => {
            move_form(studio, -1);
            Request::None
        }
        KeyCode::Down | KeyCode::Char('j') => {
            move_form(studio, 1);
            Request::None
        }
        // Enter opens whatever the selected field needs: a picker, a text
        // cursor, or a toggle it flips outright.
        KeyCode::Enter => {
            if pickers::open_for_field(studio) {
                return Request::None;
            }
            if let Some(form) = studio.form.as_mut()
                && !form.begin_typing()
            {
                form.toggle_selected();
            }
            Request::None
        }
        KeyCode::Char(' ') => {
            if let Some(form) = studio.form.as_mut() {
                form.toggle_selected();
            }
            Request::None
        }
        // Submitting is a separate key from Enter, because Enter is already
        // "open this field" and a form is usually several fields long.
        KeyCode::Char('w') => match studio.submit_form() {
            Some(message) => Request::Notice(message),
            None => Request::None,
        },
        _ => Request::None,
    }
}

/// Keys while a field's text is being edited.
fn typing_key(studio: &mut Studio, key: KeyEvent) -> Request {
    let Some(form) = studio.form.as_mut() else {
        return Request::None;
    };

    if key.modifiers.contains(KeyModifiers::CONTROL) {
        if key.code == KeyCode::Char('u') {
            form.clear_buffer();
        }
        return Request::None;
    }

    match key.code {
        KeyCode::Esc => form.cancel_typing(),
        KeyCode::Enter => {
            if !form.commit()
                && let Some(error) = form.error.clone()
            {
                return Request::Notice(error);
            }
        }
        KeyCode::Backspace => form.backspace(),
        KeyCode::Char(character) => form.push(character),
        _ => {}
    }
    Request::None
}

fn move_form(studio: &mut Studio, delta: isize) {
    if let Some(form) = studio.form.as_mut() {
        form.move_selection(delta);
    }
}
