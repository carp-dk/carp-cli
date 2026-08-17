// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! The keys that belong to one screen rather than to the app.
//!
//! [`super::handle_key`] handles what every screen shares - moving, going
//! back, quitting - and routes whatever is left here.

use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::app::App;
use crate::app::state::{Prompt, PromptKind, Status, StudyTab};
use crate::app::tasks;

pub(super) fn studies_key(app: &mut App, key: KeyEvent) {
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

pub(super) fn study_key(app: &mut App, key: KeyEvent) {
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

pub(super) fn downloads_key(app: &mut App, key: KeyEvent) {
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
