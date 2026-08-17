// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! The prompt line: the one place the app takes typed input outside the
//! protocol editor.
//!
//! A prompt swallows every keystroke while it is open, so its handling lives
//! apart from the screen bindings it temporarily replaces.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;
use crate::app::state::{Prompt, PromptKind, Status};
use crate::app::tasks;

/// Editing keys for the prompt line.
pub(super) fn handle_prompt(app: &mut App, key: KeyEvent) {
    let Some(prompt) = app.prompt.as_mut() else {
        return;
    };

    if prompt.is_confirmation() {
        handle_confirmation(app, key);
        return;
    }

    match key.code {
        KeyCode::Esc => {
            // Cancelling a live filter puts the list back as it was.
            let restore = prompt.original.clone();
            let live = prompt.kind == PromptKind::StudyFilter;
            app.prompt = None;
            if live {
                app.studies.filter = restore;
                app.studies.refilter();
            }
        }
        KeyCode::Enter => {
            let Some(prompt) = app.prompt.take() else {
                return;
            };
            apply_prompt(app, prompt);
        }
        KeyCode::Backspace => {
            prompt.value.pop();
            apply_live(app);
        }
        KeyCode::Char(character) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                // Ctrl-u clears the line, like a shell.
                if character == 'u' {
                    prompt.value.clear();
                }
            } else {
                prompt.value.push(character);
            }
            apply_live(app);
        }
        _ => {}
    }
}

/// Yes/no questions take one keystroke and nothing else.
fn handle_confirmation(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('y' | 'Y') => {
            let Some(prompt) = app.prompt.take() else {
                return;
            };
            if prompt.kind == PromptKind::ConfirmDiscardProtocol {
                app.discard_protocol();
                return;
            }
            let PromptKind::ConfirmDeleteExport { export_id, .. } = prompt.kind else {
                return;
            };
            if let Some(study) = app.study.as_ref() {
                let study_id = study.id();
                tasks::delete_export(app.client.clone(), app.sender(), study_id, export_id);
            }
        }
        KeyCode::Char('n' | 'N') | KeyCode::Esc | KeyCode::Enter => {
            app.prompt = None;
            app.status = Some(Status::info("cancelled"));
        }
        _ => {}
    }
}

/// Filtering the study list is local, so it can follow every keystroke.
fn apply_live(app: &mut App) {
    let Some(prompt) = app.prompt.as_ref() else {
        return;
    };
    if prompt.kind != PromptKind::StudyFilter {
        return;
    }
    app.studies.filter = prompt.value.clone();
    app.studies.refilter();
}

fn apply_prompt(app: &mut App, prompt: Prompt) {
    match prompt.kind {
        PromptKind::StudyFilter => {
            app.studies.filter = prompt.value;
            app.studies.refilter();
        }
        PromptKind::ConfirmDeleteExport { .. } | PromptKind::ConfirmDiscardProtocol => {}
        PromptKind::OpenProtocol => {
            let value = prompt.value.trim().to_owned();
            if !value.is_empty() {
                app.open_protocol_at(std::path::PathBuf::from(shell_expand(&value)));
            }
        }
        PromptKind::ProtocolVersionTag => {
            let value = prompt.value.trim().to_owned();
            if let Some(studio) = app.studio.as_mut()
                && !value.is_empty()
            {
                studio.version_tag = carp_protocol::VersionTag(value);
            }
        }
        PromptKind::ParticipantSearch => {
            if let Some(study) = app.study.as_mut() {
                let value = prompt.value.trim().to_owned();
                study.participants.query.search = if value.is_empty() { None } else { Some(value) };
                study.participants.query.page = 0;
            }
            app.refresh_participants();
        }
    }
}

/// Expand a leading `~` to the home directory.
///
/// Paths are typed into a prompt rather than picked from a dialog, and a
/// shell would expand this before the program ever saw it.
fn shell_expand(path: &str) -> String {
    let Some(rest) = path.strip_prefix('~') else {
        return path.to_owned();
    };
    match dirs::home_dir() {
        Some(home) => format!("{}{rest}", home.display()),
        None => path.to_owned(),
    }
}
