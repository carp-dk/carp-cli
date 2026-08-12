// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Key handling for the protocol editor.
//!
//! Keystrokes are routed by what is on top: a picker takes precedence over a
//! form, and a form over the section beneath it. That ordering is what makes
//! `Esc` mean "close the thing in front of me" at every depth, and it is why
//! each layer handles its own keys rather than the section knowing about
//! overlays.
//!
//! Bindings follow the rest of the app: `j`/`k` and the arrows move, `Enter`
//! opens, `a` adds, `e` edits, `x` removes, `Esc` goes back.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{Section, Studio, actions, pickers};

pub mod overlay;

use overlay::{form_key, picker_key};

/// What a keystroke asked the surrounding app to do.
///
/// The editor cannot save, upload or leave by itself: those need the config,
/// the API client and the app's routing. It says what it wants and the app
/// does it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    /// Nothing beyond what the editor already did.
    None,
    /// Show this in the status bar.
    Notice(String),
    /// Write the protocol to its path, asking for one if it has none.
    Save,
    /// Ask for a path and open a protocol from it.
    Open,
    /// Start a new, empty protocol.
    New,
    /// Upload the protocol to CAWS.
    Upload,
    /// Download the upstream catalogue.
    SyncCatalog,
    /// Ask for the version tag the next upload is filed under.
    SetVersionTag,
    /// Leave the editor.
    Close,
}

impl Request {
    fn notice(text: impl Into<String>) -> Self {
        Self::Notice(text.into())
    }
}

/// How far `PageUp`/`PageDown` jump.
const PAGE_STEP: isize = 10;

/// Handle one keystroke.
pub fn handle_key(studio: &mut Studio, key: KeyEvent) -> Request {
    if studio.picker.is_some() {
        return picker_key(studio, key);
    }
    if studio.form.is_some() {
        return form_key(studio, key);
    }
    section_key(studio, key)
}

/// Keys with no overlay open.
fn section_key(studio: &mut Studio, key: KeyEvent) -> Request {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('d') => {
                scroll(studio, PAGE_STEP);
                Request::None
            }
            KeyCode::Char('u') => {
                scroll(studio, -PAGE_STEP);
                Request::None
            }
            _ => Request::None,
        };
    }

    // Tab switching, shared by every section.
    match key.code {
        KeyCode::Tab | KeyCode::Right => {
            studio.section = studio.section.next();
            return Request::None;
        }
        KeyCode::BackTab | KeyCode::Left => {
            studio.section = studio.section.previous();
            return Request::None;
        }
        KeyCode::Char(digit @ '1'..='8') => {
            if let Some(section) = Section::from_index(digit as usize - '1' as usize) {
                studio.section = section;
            }
            return Request::None;
        }
        _ => {}
    }

    match key.code {
        KeyCode::Esc => Request::Close,
        KeyCode::Up | KeyCode::Char('k') => {
            scroll(studio, -1);
            Request::None
        }
        KeyCode::Down | KeyCode::Char('j') => {
            scroll(studio, 1);
            Request::None
        }
        KeyCode::PageUp => {
            scroll(studio, -PAGE_STEP);
            Request::None
        }
        KeyCode::PageDown => {
            scroll(studio, PAGE_STEP);
            Request::None
        }
        KeyCode::Home | KeyCode::Char('g') => {
            scroll(studio, isize::MIN / 2);
            Request::None
        }
        KeyCode::End | KeyCode::Char('G') => {
            scroll(studio, isize::MAX / 2);
            Request::None
        }

        KeyCode::Char('a') => notice(pickers::open_add(studio)),
        KeyCode::Char('e') => notice(actions::edit_selected(studio)),
        KeyCode::Char('x') => notice(actions::remove_selected(studio)),
        KeyCode::Char('s') => Request::Save,
        KeyCode::Char('o') => Request::Open,
        KeyCode::Char('n') => Request::New,
        KeyCode::Char('u') => Request::Upload,
        KeyCode::Char('S') => Request::SyncCatalog,
        KeyCode::Char('v') => Request::SetVersionTag,
        KeyCode::Char('r') => {
            studio.recheck();
            Request::notice("rechecked")
        }
        KeyCode::Char('z') => {
            if studio.undo() {
                Request::notice("undone")
            } else {
                Request::notice("nothing to undo")
            }
        }
        KeyCode::Char('A') => section_shift_a(studio),

        _ => section_specific(studio, key),
    }
}

/// `A` means "the other add" and differs per section.
fn section_shift_a(studio: &mut Studio) -> Request {
    match studio.section {
        Section::Overview => {
            studio.form = Some(crate::app::form::build::application_data(&studio.protocol));
            Request::None
        }
        Section::Participants => notice(actions::add_expected(studio)),
        _ => Request::None,
    }
}

/// Keys that only exist in one section.
fn section_specific(studio: &mut Studio, key: KeyEvent) -> Request {
    match studio.section {
        Section::Tasks => match key.code {
            KeyCode::Enter => notice(actions::open_survey(studio)),
            KeyCode::Char('m') => notice(actions::add_measure(studio)),
            KeyCode::Char('M') => notice(actions::edit_selected_measure(studio)),
            KeyCode::Char('X') => notice(actions::remove_selected_measure(studio)),
            _ => Request::None,
        },
        Section::Survey => match key.code {
            KeyCode::Char('J') => notice(actions::move_step(studio, 1)),
            KeyCode::Char('K') => notice(actions::move_step(studio, -1)),
            _ => Request::None,
        },
        Section::Participants => match key.code {
            KeyCode::Char('E') => notice(actions::edit_selected_expected(studio)),
            KeyCode::Char('X') => notice(actions::remove_selected_expected(studio)),
            _ => Request::None,
        },
        Section::Catalog => match key.code {
            KeyCode::Enter => notice(use_selected_template(studio)),
            _ => Request::None,
        },
        Section::Overview => match key.code {
            KeyCode::Char('D') => {
                studio.form = Some(crate::app::form::build::data_end_point(&studio.protocol));
                Request::None
            }
            _ => Request::None,
        },
        _ => Request::None,
    }
}

/// Start from the template under the cursor in the catalogue pane.
fn use_selected_template(studio: &mut Studio) -> Option<String> {
    let index = studio.lists.templates.selected()?;
    let study = studio.catalog.templates.get(index)?.study.clone();
    studio.creating = Some(pickers::Creating::Template);
    studio.picker = Some(crate::app::form::picker::Picker::new(
        "start from a study",
        crate::app::form::picker::PickerKind::Create,
        vec![crate::app::form::picker::Row::new(&study, &study, "")],
        &study,
    ));
    pickers::resolve(studio)
}

fn notice(message: Option<String>) -> Request {
    match message {
        Some(message) => Request::Notice(message),
        None => Request::None,
    }
}

/// Move the cursor of the current section's list.
fn scroll(studio: &mut Studio, delta: isize) {
    let survey_task = studio.survey_task_name();
    let checks = studio.diagnostics.len();
    let templates = studio.catalog.templates.len();
    studio.lists.move_in(
        studio.section,
        &studio.protocol,
        survey_task.as_deref(),
        checks,
        templates,
        delta,
    );
}

#[cfg(test)]
mod editing_tests;
#[cfg(test)]
pub mod tests;
