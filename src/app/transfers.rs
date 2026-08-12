// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Downloads, and the things this app opens outside the terminal.

use crate::app::App;
use crate::api::endpoints::{exports, files};
use crate::app::state::{Route, Status, StudyState};
use crate::app::tasks;

impl App {

    /// Download the selected export archive.
    pub fn download_selected_export(&mut self) {
        let Some(study) = self.study.as_ref() else {
            return;
        };
        let study_id = study.id();
        let Some(export) = study.selected_export() else {
            return;
        };
        if !export.status.is_downloadable() {
            self.status = Some(Status::info(format!(
                "export is {} - only available exports can be downloaded",
                export.status.label()
            )));
            return;
        }

        let label = export.display_name();
        let path = exports::download_path(&study_id, &export.id);
        let directory = self.study_download_dir(&study_id);
        let job_id = self.downloads.enqueue(label.clone(), Some(study_id));
        tasks::download(
            self.client.clone(),
            self.tx.clone(),
            path,
            directory,
            label,
            job_id,
        );
        self.status = Some(Status::info("download started - press d to watch it"));
    }

    /// Download the selected study file.
    pub fn download_selected_file(&mut self) {
        let Some(study) = self.study.as_ref() else {
            return;
        };
        let study_id = study.id();
        let Some(file) = study.selected_file() else {
            return;
        };

        let label = file.download_name().to_owned();
        let path = files::download_path(&study_id, file.id);
        let directory = self.study_download_dir(&study_id);
        let job_id = self.downloads.enqueue(label.clone(), Some(study_id));
        tasks::download(
            self.client.clone(),
            self.tx.clone(),
            path,
            directory,
            label,
            job_id,
        );
        self.status = Some(Status::info("download started - press d to watch it"));
    }

    /// Open the highlighted study in the CARP web portal.
    ///
    /// No token is handed over: the CLI signed in through this browser, so
    /// Keycloak's session cookie is already there and the portal picks the
    /// session up by itself.
    pub fn open_study_in_browser(&mut self) {
        let study_id = match self.route {
            Route::Studies => self
                .studies
                .selected()
                .map(|study| study.study_id.to_string()),
            _ => self.study.as_ref().map(StudyState::id),
        };
        let Some(study_id) = study_id else {
            return;
        };

        let url = self.portal.study_url(&study_id);
        match webbrowser::open(url.as_str()) {
            Ok(()) => {
                // Show the address: the portal path is a convention, not
                // something the API states, so a wrong guess must be visible.
                let mut message = format!("opened {url}");
                if !self.portal.is_resolved() {
                    message.push_str(" · set CARP_PORTAL_URL if that is not your portal");
                }
                self.status = Some(Status::info(message));
            }
            Err(error) => {
                self.status = Some(Status::error(format!("could not open a browser: {error}")));
            }
        }
    }

    /// Open the folder holding the selected download in the desktop file
    /// manager, so the archive can be handed to whatever analyses it.
    pub fn reveal_selected_download(&mut self) {
        let selected = self.downloads_table.selected().unwrap_or(0);
        let Some(job) = self.downloads.jobs().get(selected) else {
            return;
        };
        let Some(path) = job.path.clone() else {
            self.status = Some(Status::info(
                "this transfer has not written a file yet".to_owned(),
            ));
            return;
        };

        let folder = path.parent().unwrap_or(&path).to_path_buf();
        match url::Url::from_file_path(&folder).map(|url| webbrowser::open(url.as_str())) {
            Ok(Ok(())) => {
                self.status = Some(Status::info(format!("opened {}", folder.display())));
            }
            Ok(Err(error)) => {
                self.status = Some(Status::error(format!("could not open the folder: {error}")));
            }
            Err(()) => {
                self.status = Some(Status::error(format!(
                    "not a openable path: {}",
                    folder.display()
                )));
            }
        }
    }

    /// True while any request is in flight, for the header indicator.
    pub fn is_busy(&self) -> bool {
        self.studies.loading
            || self.study.as_ref().is_some_and(|study| {
                study.details_loading
                    || study.participants.loading
                    || study.files_loading
                    || study.exports_loading
            })
    }

    /// Downloads are grouped per study, named after the study when possible.
    fn study_download_dir(&self, study_id: &str) -> std::path::PathBuf {
        let name = self
            .study
            .as_ref()
            .map(|study| study.study.name.clone())
            .filter(|name| !name.trim().is_empty())
            .map(|name| {
                name.chars()
                    .map(|c| {
                        if c.is_alphanumeric() || c == '-' {
                            c
                        } else {
                            '_'
                        }
                    })
                    .collect::<String>()
            })
            .unwrap_or_else(|| study_id.to_owned());
        self.config.download_dir.join(name)
    }
}
