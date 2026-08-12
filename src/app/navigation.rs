// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Moving between screens, and loading what a screen needs.

use crate::app::App;
use crate::api::models::{ParticipantSummary, StudyOverview};
use crate::app::state::{ParticipantState, Route, StudyState, StudyTab};
use crate::app::{state, tasks};

impl App {

    pub fn open_study(&mut self, study: StudyOverview) {
        let study_id = study.study_id.to_string();
        let mut state = StudyState::new(study);
        state.details_loading = true;
        self.study = Some(state);
        self.route = Route::Study;

        tasks::load_study_details(self.client.clone(), self.tx.clone(), study_id.clone());
        tasks::load_cached_participants(self.cache.clone(), self.tx.clone(), study_id);
    }

    pub fn open_participant(&mut self, participant: ParticipantSummary) {
        let Some(study) = self.study.as_ref() else {
            return;
        };
        let group = study.group_for(&participant.participant_id).cloned();
        self.participant = Some(ParticipantState {
            study: study.study.clone(),
            participant,
            group,
        });
        self.route = Route::Participant;
    }

    /// Leave the current screen.
    pub fn back(&mut self) {
        match self.route {
            Route::Studies => {}
            Route::Study => {
                self.study = None;
                self.route = Route::Studies;
            }
            Route::Participant => {
                self.participant = None;
                self.route = Route::Study;
            }
            // The editor asks before discarding unsaved work, so leaving it
            // goes through the same request path a key press would.
            Route::Studio => self.handle_studio_request(crate::studio::input::Request::Close),
            Route::Downloads => {
                self.route = if self.participant.is_some() {
                    Route::Participant
                } else if self.study.is_some() {
                    Route::Study
                } else {
                    Route::Studies
                };
            }
        }
    }

    pub fn show_downloads(&mut self) {
        self.route = Route::Downloads;
        state::clamp_selection(&mut self.downloads_table, self.downloads.jobs().len());
    }

    /// Load whatever the newly selected tab needs, once.
    pub fn select_tab(&mut self, tab: StudyTab) {
        if let Some(study) = self.study.as_mut() {
            study.tab = tab;
        }
        self.ensure_tab_loaded();
    }

    /// Load whatever the visible tab needs, once.
    pub fn ensure_tab_loaded(&mut self) {
        let Some(study) = self.study.as_ref() else {
            return;
        };
        let tab = study.tab;
        let needs_details = !study.details_loaded && !study.details_loading;
        let needs_participants = !study.participants.loaded && !study.participants.loading;
        let needs_files = !study.files_loaded && !study.files_loading;
        let needs_exports = !study.exports_loaded && !study.exports_loading;

        match tab {
            StudyTab::Overview | StudyTab::Staff => {
                if needs_details {
                    self.refresh_study_details();
                }
            }
            StudyTab::Participants => {
                if needs_participants {
                    self.refresh_participants();
                }
            }
            StudyTab::Deployments => {
                if needs_details {
                    self.refresh_study_details();
                }
                // A deployment belongs to a participant group; without the
                // participants loaded it could only show their ids.
                if needs_participants {
                    self.refresh_participants();
                }
            }
            StudyTab::Files => {
                if needs_files {
                    self.refresh_files();
                }
            }
            StudyTab::Exports => {
                if needs_exports {
                    self.refresh_exports();
                }
            }
        }
    }
}
