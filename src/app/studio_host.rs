// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Where the protocol editor meets the rest of the app.
//!
//! [`crate::studio`] holds and edits a protocol; it cannot save, open, upload
//! or sync by itself, because those need the config, the API client and the
//! signed-in account. It says what it wants as a
//! [`crate::studio::input::Request`], and this module carries it out.
//!
//! Keeping the two apart means the editor is testable without a network or a
//! filesystem, which is why its own tests can drive it entirely through keys.

use std::path::PathBuf;

use carp_protocol::version::UploadCheck;

use crate::app::state::{Prompt, PromptKind, Route, Status};
use crate::app::{App, studio_tasks};
use crate::studio::input::Request;
use crate::studio::{CatalogState, Studio};

impl App {
    /// Open the protocol editor, creating one if it is not already open.
    pub fn open_studio(&mut self) {
        if self.studio.is_none() {
            // The owner id is a UUID CAWS replaces on upload. The signed-in
            // account's own id is not exposed by the API, so a fresh one is
            // used and the Overview tab shows it for editing.
            let owner = uuid::Uuid::new_v4().to_string();
            let mut studio = Studio::new(owner);
            studio.catalog = std::mem::take(&mut self.catalog);
            studio.snapshot = self.catalog_snapshot.take();
            studio.catalog_state = if studio.catalog.is_empty() {
                CatalogState::Absent
            } else {
                CatalogState::Ready
            };
            self.studio = Some(studio);
        }
        self.route = Route::Studio;
    }

    /// Carry out what a keystroke in the editor asked for.
    pub fn handle_studio_request(&mut self, request: Request) {
        match request {
            Request::None => {}
            Request::Notice(message) => self.status = Some(Status::info(message)),
            Request::Close => self.leave_studio(),
            Request::Save => self.save_protocol(),
            Request::Open => {
                self.prompt = Some(Prompt::new(PromptKind::OpenProtocol, String::new()));
            }
            Request::New => self.new_protocol(),
            Request::Upload => self.upload_protocol(),
            Request::SyncCatalog => self.sync_catalog(),
            Request::SetVersionTag => {
                let current = self
                    .studio
                    .as_ref()
                    .map(|studio| studio.version_tag.to_string())
                    .unwrap_or_default();
                self.prompt = Some(Prompt::new(PromptKind::ProtocolVersionTag, current));
            }
        }
    }

    /// Leave the editor, warning once when there is unsaved work.
    fn leave_studio(&mut self) {
        let dirty = self.studio.as_ref().is_some_and(|studio| studio.dirty);
        if dirty {
            self.prompt = Some(Prompt::confirm(PromptKind::ConfirmDiscardProtocol));
            return;
        }
        self.leave_to_studies();
    }

    /// Leave without saving, once the warning has been accepted.
    pub fn discard_protocol(&mut self) {
        self.studio = None;
        self.leave_to_studies();
        self.status = Some(Status::info("left without saving"));
    }

    /// Land on the study list, loading it if this session never has.
    ///
    /// `carp protocol` skips the startup load, so arriving here may be the
    /// first time the list is wanted - and the first time a session is needed.
    pub(super) fn leave_to_studies(&mut self) {
        self.route = Route::Studies;
        if self.studies.items.is_empty() && !self.studies.loading {
            self.refresh_studies();
        }
    }

    /// Write the protocol to its path, choosing one when it has none.
    fn save_protocol(&mut self) {
        let Some(studio) = self.studio.as_ref() else {
            return;
        };

        let path = studio.path.clone().unwrap_or_else(|| {
            crate::protocol_file::default_path(&studio.protocol, &self.config.download_dir)
        });
        let protocol = studio.protocol.clone();
        studio_tasks::save_protocol(protocol, path, self.sender());
    }

    /// Start a blank protocol, warning when the current one is unsaved.
    fn new_protocol(&mut self) {
        if self.studio.as_ref().is_some_and(|studio| studio.dirty) {
            self.status = Some(Status::error(
                "save the current protocol first, or press Esc to discard it",
            ));
            return;
        }

        let Some(studio) = self.studio.as_mut() else {
            return;
        };
        let owner = studio.protocol.owner_id.clone();
        let catalog = std::mem::take(&mut studio.catalog);
        let snapshot = studio.snapshot.take();
        let state = studio.catalog_state.clone();

        let mut fresh = Studio::new(owner);
        fresh.catalog = catalog;
        fresh.snapshot = snapshot;
        fresh.catalog_state = state;
        *studio = fresh;

        self.status = Some(Status::info("started a new protocol"));
    }

    /// Read a protocol from `path` into the editor.
    pub fn open_protocol_at(&mut self, path: PathBuf) {
        studio_tasks::open_protocol(path, self.sender());
    }

    /// Send the protocol to CAWS.
    ///
    /// Refused before it leaves when [`UploadCheck`] finds a blocker: a
    /// rejected upload costs a round trip and tells the user less than the
    /// local check does.
    fn upload_protocol(&mut self) {
        let Some(studio) = self.studio.as_mut() else {
            return;
        };

        let check = UploadCheck::run(&studio.protocol);
        if !check.is_ready() {
            let count = check.blockers.len();
            self.status = Some(Status::error(format!(
                "not ready to upload: {} ({count} problem{} - see the Checks tab)",
                check.blockers[0],
                if count == 1 { "" } else { "s" }
            )));
            return;
        }

        // The protocol's `version` counter is CARP's, not the client's: every
        // upstream protocol carries 0, including ones on their third published
        // version. Versioning is done by tag, so the document is sent exactly
        // as it sits on disk and only the tag moves - and only once CAWS has
        // accepted it, so a failed upload leaves nothing changed.
        let tag = studio.version_tag.to_string();
        let protocol = studio.protocol.clone();
        studio_tasks::upload_protocol(self.client.clone(), protocol, tag.clone(), self.sender());
        self.status = Some(Status::info(format!("uploading as {tag}…")));
    }

    /// Download the upstream catalogue.
    fn sync_catalog(&mut self) {
        if let Some(studio) = self.studio.as_mut() {
            studio.catalog_state = CatalogState::Syncing;
        }
        studio_tasks::sync_catalog(self.config.data_dir.clone(), self.sender());
        self.status = Some(Status::info("downloading the upstream studies…"));
    }

    /// Store a catalogue that finished loading or syncing.
    ///
    /// The editor may not be open yet, so the catalogue is parked on the app
    /// and handed over when it opens.
    pub fn set_catalog(
        &mut self,
        catalog: carp_catalog::Catalog,
        snapshot: Option<carp_catalog::Snapshot>,
    ) {
        match self.studio.as_mut() {
            Some(studio) => {
                studio.catalog = catalog;
                if snapshot.is_some() {
                    studio.snapshot = snapshot;
                }
                studio.catalog_state = CatalogState::Ready;
                // Whatever update prompted this sync has now been applied.
                studio.update_available = None;
                studio.sync_selection();
            }
            None => {
                self.catalog = catalog;
                if snapshot.is_some() {
                    self.catalog_snapshot = snapshot;
                }
            }
        }
    }

    /// Record that the catalogue could not be loaded or synced.
    pub fn set_catalog_failed(&mut self, error: String) {
        if let Some(studio) = self.studio.as_mut() {
            studio.catalog_state = CatalogState::Failed(error.clone());
        }
        self.status = Some(Status::error(format!("catalogue: {error}")));
    }

    /// Record that upstream has moved past the stored catalogue.
    pub fn set_catalog_update(&mut self, commit: carp_catalog::Commit) {
        if let Some(studio) = self.studio.as_mut() {
            studio.update_available = Some(commit);
        } else {
            self.catalog_update = Some(commit);
        }
    }

    /// Note a completed save.
    pub fn protocol_saved(&mut self, path: PathBuf) {
        if let Some(studio) = self.studio.as_mut() {
            studio.dirty = false;
            studio.path = Some(path.clone());
        }
        self.status = Some(Status::info(format!("saved {}", path.display())));
    }

    /// Put a protocol read from disk into the editor.
    pub fn protocol_opened(&mut self, protocol: carp_protocol::StudyProtocol, path: PathBuf) {
        let (catalog, snapshot, state, update) = match self.studio.as_mut() {
            Some(studio) => (
                std::mem::take(&mut studio.catalog),
                studio.snapshot.take(),
                studio.catalog_state.clone(),
                studio.update_available.take(),
            ),
            None => (
                std::mem::take(&mut self.catalog),
                self.catalog_snapshot.take(),
                CatalogState::Ready,
                self.catalog_update.take(),
            ),
        };

        let name = protocol.name.clone();
        let mut studio = Studio::opened(protocol, Some(path));
        studio.catalog = catalog;
        studio.snapshot = snapshot;
        studio.catalog_state = if studio.catalog.is_empty() {
            CatalogState::Absent
        } else {
            state
        };
        studio.update_available = update;

        self.studio = Some(studio);
        self.route = Route::Studio;
        self.status = Some(Status::info(format!("opened {name}")));
    }

    /// Note the result of an upload.
    ///
    /// A successful upload moves the version tag on, so the next one does not
    /// collide with the tag just used.
    pub fn protocol_uploaded(&mut self, message: String, stored: bool) {
        if stored
            && let Some(studio) = self.studio.as_mut()
            && let Some(next) = studio.version_tag.next(carp_protocol::version::Bump::Minor)
        {
            studio.version_tag = next;
        }

        self.status = Some(if stored {
            Status::info(message)
        } else {
            Status::error(message)
        });
    }
}
