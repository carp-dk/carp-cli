// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Key handling. Input only mutates state or starts a task; it never performs
//! I/O itself.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;
use crate::app::state::{Route, StudyTab, move_selection};

pub mod prompt;
pub mod screens;

use prompt::handle_prompt;
use screens::{downloads_key, studies_key, study_key};

/// How far `PageUp`/`PageDown` jump.
const PAGE_STEP: isize = 10;

pub fn handle_key(app: &mut App, key: KeyEvent) {
    // A prompt swallows everything while it is open.
    if app.prompt.is_some() {
        handle_prompt(app, key);
        return;
    }

    // So does the help overlay.
    if app.show_help {
        if matches!(
            key.code,
            KeyCode::Char('?') | KeyCode::Esc | KeyCode::Char('q')
        ) {
            app.show_help = false;
        }
        return;
    }

    // Any keystroke acknowledges the last message, so the key hints come back.
    app.status = None;

    // The editor is modal: while it is open every key belongs to it, including
    // the ones the rest of the app uses for its own navigation.
    if app.route == Route::Studio && app.studio.is_some() {
        let request = {
            let studio = app.studio.as_mut().expect("checked above");
            crate::studio::input::handle_key(studio, key)
        };
        // Ctrl-C still quits, so the app cannot be trapped in the editor.
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            app.should_quit = true;
            return;
        }
        app.handle_studio_request(request);
        return;
    }

    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('c') => app.should_quit = true,
            KeyCode::Char('d') => scroll(app, PAGE_STEP),
            KeyCode::Char('u') => scroll(app, -PAGE_STEP),
            _ => {}
        }
        return;
    }

    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('?') => app.show_help = true,
        KeyCode::Esc => app.back(),
        KeyCode::Char('r') => app.refresh(),
        KeyCode::Char('d') => app.show_downloads(),
        KeyCode::Char('P') => app.open_studio(),
        // `o` opens things outside the terminal: a study in the browser, or
        // on the transfer list the folder a download landed in.
        KeyCode::Char('o') if app.route != Route::Downloads => app.open_study_in_browser(),
        KeyCode::Up | KeyCode::Char('k') => scroll(app, -1),
        KeyCode::Down | KeyCode::Char('j') => scroll(app, 1),
        KeyCode::PageUp => scroll(app, -PAGE_STEP),
        KeyCode::PageDown => scroll(app, PAGE_STEP),
        KeyCode::Home | KeyCode::Char('g') => scroll(app, isize::MIN / 2),
        KeyCode::End | KeyCode::Char('G') => scroll(app, isize::MAX / 2),
        _ => match app.route {
            Route::Studies => studies_key(app, key),
            Route::Study => study_key(app, key),
            Route::Participant => {}
            Route::Downloads => downloads_key(app, key),
            // The editor is modal and owns every key while it is open, which
            // is handled before this match is reached.
            Route::Studio => {}
        },
    }
}

/// Move the cursor of whichever table the current screen shows.
fn scroll(app: &mut App, delta: isize) {
    match app.route {
        Route::Studies => {
            let len = app.studies.len();
            move_selection(&mut app.studies.table, len, delta);
        }
        Route::Downloads => {
            let len = app.downloads.jobs().len();
            move_selection(&mut app.downloads_table, len, delta);
        }
        Route::Participant | Route::Studio => {}
        Route::Study => {
            let Some(study) = app.study.as_mut() else {
                return;
            };
            match study.tab {
                StudyTab::Overview => {}
                StudyTab::Participants => {
                    let len = study.participants.items.len();
                    move_selection(&mut study.participants.table, len, delta);
                }
                StudyTab::Deployments => {
                    let len = study.groups().groups.len();
                    move_selection(&mut study.groups_table, len, delta);
                }
                StudyTab::Staff => {
                    let len = study.staff().len();
                    move_selection(&mut study.staff_table, len, delta);
                }
                StudyTab::Files => {
                    let len = study.files.len();
                    move_selection(&mut study.files_table, len, delta);
                }
                StudyTab::Exports => {
                    let len = study.exports.len();
                    move_selection(&mut study.exports_table, len, delta);
                }
            }
        }
    }
}
