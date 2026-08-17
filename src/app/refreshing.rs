// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Reloading what a screen shows.

use crate::app::App;
use crate::app::state::{Route, StudyTab};
use crate::app::tasks;

impl App {
    /// Reload whatever the current screen shows.
    pub fn refresh(&mut self) {
        match self.route {
            Route::Studies => self.refresh_studies(),
            Route::Study => match self.study.as_ref().map(|study| study.tab) {
                Some(StudyTab::Participants) => self.refresh_participants(),
                Some(StudyTab::Files) => self.refresh_files(),
                Some(StudyTab::Exports) => self.refresh_exports(),
                Some(StudyTab::Overview | StudyTab::Staff | StudyTab::Deployments) => {
                    self.refresh_study_details()
                }
                None => {}
            },
            Route::Participant => self.refresh_participants(),
            Route::Downloads => {
                tasks::load_download_history(self.cache.clone(), self.tx.clone());
            }
            Route::Studio => {
                if let Some(studio) = self.studio.as_mut() {
                    studio.recheck();
                }
            }
        }
    }

    pub fn refresh_studies(&mut self) {
        self.studies.loading = true;
        tasks::load_studies(self.client.clone(), self.cache.clone(), self.tx.clone());
    }

    pub fn refresh_study_details(&mut self) {
        let Some(study) = self.study.as_mut() else {
            return;
        };
        study.details_loading = true;
        let study_id = study.id();
        tasks::load_study_details(self.client.clone(), self.tx.clone(), study_id);
    }

    pub fn refresh_participants(&mut self) {
        let Some(study) = self.study.as_mut() else {
            return;
        };
        study.participants.loading = true;
        let study_id = study.id();
        let query = study.participants.query.clone();
        tasks::load_participants(
            self.client.clone(),
            self.cache.clone(),
            self.tx.clone(),
            study_id,
            query,
        );
    }

    pub fn refresh_files(&mut self) {
        let Some(study) = self.study.as_mut() else {
            return;
        };
        study.files_loading = true;
        let study_id = study.id();
        tasks::load_files(self.client.clone(), self.tx.clone(), study_id);
    }

    /// An export that is being built changes state server side; keep the list
    /// current while the user is watching it.
    pub(super) fn poll_pending_exports(&mut self) {
        let pending = self.study.as_ref().is_some_and(|study| {
            study.tab == StudyTab::Exports
                && !study.exports_loading
                && study
                    .exports
                    .iter()
                    .any(|export| export.status.is_pending())
        });
        if pending {
            self.refresh_exports();
        }
    }

    pub fn refresh_exports(&mut self) {
        let Some(study) = self.study.as_mut() else {
            return;
        };
        study.exports_loading = true;
        let study_id = study.id();
        tasks::load_exports(self.client.clone(), self.tx.clone(), study_id);
    }
}
