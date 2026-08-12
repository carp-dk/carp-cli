// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Key handling. Input only mutates state or starts a task; it never performs
//! I/O itself.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;
use crate::app::state::{Prompt, PromptKind, Route, Status, StudyTab, move_selection};
use crate::app::tasks;

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

fn studies_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            if let Some(study) = app.studies.selected().cloned() {
                app.open_study(study);
            }
        }
        KeyCode::Char('/') => {
            app.prompt = Some(Prompt::new(
                PromptKind::StudyFilter,
                app.studies.filter.clone(),
            ));
        }
        KeyCode::Char('c') => {
            app.studies.filter.clear();
            app.studies.refilter();
        }
        KeyCode::Char('s') => {
            app.studies.sort = app.studies.sort.next();
            app.studies.refilter();
            app.status = Some(Status::info(format!(
                "sorted by {}",
                app.studies.sort.label()
            )));
        }
        _ => {}
    }
}

fn study_key(app: &mut App, key: KeyEvent) {
    // Tab switching is shared by every tab.
    match key.code {
        KeyCode::Tab | KeyCode::Right => {
            let next = app.study.as_ref().map(|study| study.tab.next());
            if let Some(tab) = next {
                app.select_tab(tab);
            }
            return;
        }
        KeyCode::BackTab | KeyCode::Left => {
            let previous = app.study.as_ref().map(|study| study.tab.previous());
            if let Some(tab) = previous {
                app.select_tab(tab);
            }
            return;
        }
        KeyCode::Char(digit @ '1'..='6') => {
            let index = digit as usize - '1' as usize;
            if let Some(tab) = StudyTab::from_index(index) {
                app.select_tab(tab);
            }
            return;
        }
        _ => {}
    }

    let Some(tab) = app.study.as_ref().map(|study| study.tab) else {
        return;
    };
    match tab {
        StudyTab::Overview | StudyTab::Staff | StudyTab::Deployments => {}
        StudyTab::Participants => participants_key(app, key),
        StudyTab::Files => files_key(app, key),
        StudyTab::Exports => exports_key(app, key),
    }
}

fn participants_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            let selected = app
                .study
                .as_ref()
                .and_then(|study| study.participants.selected().cloned());
            if let Some(participant) = selected {
                app.open_participant(participant);
            }
        }
        KeyCode::Char('/') => {
            let current = app
                .study
                .as_ref()
                .and_then(|study| study.participants.query.search.clone())
                .unwrap_or_default();
            app.prompt = Some(Prompt::new(PromptKind::ParticipantSearch, current));
        }
        KeyCode::Char('n') => change_page(app, 1),
        KeyCode::Char('p') => change_page(app, -1),
        KeyCode::Char('s') => {
            if let Some(study) = app.study.as_mut() {
                let query = &mut study.participants.query;
                query.sort_by = query.sort_by.toggled();
                query.page = 0;
            }
            app.refresh_participants();
        }
        KeyCode::Char('S') => {
            if let Some(study) = app.study.as_mut() {
                let query = &mut study.participants.query;
                query.sort_direction = query.sort_direction.toggled();
                query.page = 0;
            }
            app.refresh_participants();
        }
        KeyCode::Char('f') => {
            if let Some(study) = app.study.as_mut() {
                let query = &mut study.participants.query;
                // all -> deployed -> not deployed -> all
                query.deployed = match query.deployed {
                    None => Some(true),
                    Some(true) => Some(false),
                    Some(false) => None,
                };
                query.page = 0;
            }
            app.refresh_participants();
        }
        _ => {}
    }
}

fn change_page(app: &mut App, delta: i64) {
    let Some(study) = app.study.as_mut() else {
        return;
    };
    let pages = study.participants.page_count();
    let current = i64::from(study.participants.query.page);
    let next = (current + delta).clamp(0, i64::from(pages) - 1) as u32;
    if next == study.participants.query.page {
        return;
    }
    study.participants.query.page = next;
    app.refresh_participants();
}

fn files_key(app: &mut App, key: KeyEvent) {
    if matches!(key.code, KeyCode::Enter) {
        app.download_selected_file();
    }
}

fn exports_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => app.download_selected_export(),
        KeyCode::Char('n') => {
            if let Some(study) = app.study.as_ref() {
                let study_id = study.id();
                tasks::request_export(app.client.clone(), app.sender(), study_id);
                app.status = Some(Status::info("requesting a study data export"));
            }
        }
        // Deleting an export is not undoable, so ask first.
        KeyCode::Char('x') => {
            let selected = app.study.as_ref().and_then(|study| {
                study
                    .selected_export()
                    .map(|export| (export.id.clone(), export.display_name()))
            });
            if let Some((export_id, name)) = selected {
                app.prompt = Some(Prompt::confirm(PromptKind::ConfirmDeleteExport {
                    export_id,
                    name,
                }));
            }
        }
        _ => {}
    }
}

fn downloads_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('c') => {
            app.downloads.clear_finished();
            crate::app::state::clamp_selection(
                &mut app.downloads_table,
                app.downloads.jobs().len(),
            );
        }
        KeyCode::Char('o') | KeyCode::Enter => app.reveal_selected_download(),
        _ => {}
    }
}

pub mod prompt;

use prompt::handle_prompt;
