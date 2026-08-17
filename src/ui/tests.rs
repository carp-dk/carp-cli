// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`].

use std::sync::Arc;

use ratatui::Terminal;
use ratatui::backend::TestBackend;

use super::*;
use crate::app::state::{ParticipantState, Prompt, PromptKind, StudyState, StudyTab};
use crate::db::Cache;
use carp_client::api::CarpClient;
use carp_client::api::models::{
    Account, Export, ExportStatus, ParticipantGroupStatus, ParticipantSummary, StudyFile,
    StudyOverview,
};
use carp_client::auth::Authenticator;
use carp_client::config::Config;
use carp_client::fixtures::{PARTICIPANT_GROUP_MEMBER_ID, PARTICIPANT_GROUP_STATUS};

fn app() -> App {
    let config = Config {
        server: url::Url::parse("https://dev.carp.dk").unwrap(),
        realm: "Carp".to_owned(),
        client_id: "carp-cli".to_owned(),
        data_dir: std::env::temp_dir().join("carp-cli-tests"),
        download_dir: std::env::temp_dir().join("carp-cli-tests/downloads"),
        portal_url: None,
        portal_study_path: carp_client::config::DEFAULT_PORTAL_STUDY_PATH.to_owned(),
    };
    let authenticator = Arc::new(Authenticator::new(&config).unwrap());
    let client = CarpClient::new(&config, authenticator).unwrap();
    App::new(config, client, Cache::disabled(), Some("tester".to_owned()))
}

fn study() -> StudyOverview {
    StudyOverview {
        name: "Sleep and mood".to_owned(),
        description: Some("A feasibility study".to_owned()),
        created_by: Some("researcher@dtu.dk".to_owned()),
        ..StudyOverview::default()
    }
}

/// A study with one row in every tab, so the panels render with content
/// rather than only their empty states.
fn populated_study() -> StudyState {
    let mut state = StudyState::new(study());
    state.participants.set_items(
        vec![ParticipantSummary {
            participant_id: PARTICIPANT_GROUP_MEMBER_ID.to_owned(),
            first_name: Some("Ada".to_owned()),
            last_name: Some("Lovelace".to_owned()),
            account_identity: Some("ada@dtu.dk".to_owned()),
            deployed: true,
            carp_user: true,
            ..ParticipantSummary::default()
        }],
        1,
        false,
    );
    state.researchers = vec![Account {
        first_name: Some("Grace".to_owned()),
        last_name: Some("Hopper".to_owned()),
        email: Some("grace@dtu.dk".to_owned()),
        ..Account::default()
    }];
    state.files = vec![StudyFile {
        id: 7,
        file_name: "upload-7.json".to_owned(),
        original_name: "sleep-diary.json".to_owned(),
        relative_path: "studies/x/upload-7.json".to_owned(),
        metadata: Some(serde_json::json!({ "task": "diary" })),
        ..StudyFile::default()
    }];
    state.files_loaded = true;
    state.exports = vec![
        Export {
            id: "8f14e45f-ea1e-4a6f-8f2c-1b0d9a2e3c44".to_owned(),
            status: ExportStatus::Available,
            file_name: "study-data.zip".to_owned(),
            ..Export::default()
        },
        // A just-requested export: no archive name yet.
        Export {
            id: "c9f0f895-fb98-4b3a-9d0a-2a1c3e4f5b66".to_owned(),
            status: ExportStatus::InProgress,
            ..Export::default()
        },
    ];
    state.exports_loaded = true;
    state.set_groups(
        serde_json::from_str::<ParticipantGroupStatus>(PARTICIPANT_GROUP_STATUS).unwrap(),
    );
    state.details_loaded = true;
    state
}

/// Build the app in the state named by `label`, for the visual dump.
pub(super) fn screen(label: &str) -> App {
    let mut app = app();
    app.studies.set_items(vec![study(), study()], false);
    let mut state = populated_study();
    state.tab = match label {
        "overview" => StudyTab::Overview,
        "participants" => StudyTab::Participants,
        "deployments" => StudyTab::Deployments,
        "staff" => StudyTab::Staff,
        "files" => StudyTab::Files,
        "exports" => StudyTab::Exports,
        _ => StudyTab::Overview,
    };
    app.show_help = label == "help";
    let participant = state
        .participants
        .items
        .first()
        .cloned()
        .unwrap_or_default();
    let group = state.group_for(&participant.participant_id).cloned();
    app.participant = Some(crate::app::state::ParticipantState {
        study: study(),
        participant,
        group,
    });
    app.route = match label {
        "studies" | "help" => Route::Studies,
        "participant" => Route::Participant,
        _ => Route::Study,
    };
    app.study = Some(state);
    app
}

/// Every screen must render at a normal and at a cramped terminal size.
#[test]
fn every_screen_renders() {
    for (width, height) in [(160, 48), (110, 26), (MIN_WIDTH, MIN_HEIGHT)] {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();

        for route in [
            Route::Studies,
            Route::Study,
            Route::Participant,
            Route::Downloads,
        ] {
            for tab in StudyTab::ALL {
                let mut app = app();
                app.route = route;
                app.studies.set_items(vec![study()], false);

                let mut state = populated_study();
                state.tab = tab;
                app.study = Some(state);
                let participant = app
                    .study
                    .as_ref()
                    .and_then(|study| study.participants.items.first().cloned())
                    .unwrap_or_default();
                let group = app
                    .study
                    .as_ref()
                    .and_then(|study| study.group_for(&participant.participant_id).cloned());
                app.participant = Some(ParticipantState {
                    study: study(),
                    participant,
                    group,
                });

                terminal.draw(|frame| draw(frame, &mut app)).unwrap();
            }
        }
    }
}

/// Below the minimum size the app says so rather than drawing a broken
/// layout.
#[test]
fn a_tiny_terminal_is_told_so() {
    let mut terminal = Terminal::new(TestBackend::new(30, 8)).unwrap();
    let mut app = app();
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();

    let rendered: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect();
    assert!(rendered.contains("terminal too small"), "{rendered}");
}

/// The help overlay and the prompt draw on top of the study list.
#[test]
fn overlays_render() {
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    let mut app = app();
    app.studies.set_items(vec![study()], false);
    app.show_help = true;
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();

    app.show_help = false;
    app.prompt = Some(Prompt::new(PromptKind::StudyFilter, "sleep".to_owned()));
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
}
