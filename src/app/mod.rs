//! Application state and the main loop.
//!
//! The loop is a plain message pump: draw, wait for the next [`Message`],
//! apply it, repeat. All I/O happens in tasks that send messages back, so
//! rendering never blocks.

pub mod form;
pub mod input;
pub mod message;
pub mod state;
pub mod studio_host;
pub mod studio_tasks;
pub mod tasks;

use color_eyre::Result;
use ratatui::DefaultTerminal;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use crate::api::CarpClient;
use crate::app::message::{LoadTarget, Message};
use crate::app::state::{
    ParticipantState, Prompt, Route, Status, StudiesState, StudyState,
};
use crate::config::Config;
use crate::db::{Cache, DownloadRecord};
use crate::download::DownloadManager;
use crate::portal::Portal;
use crate::tui;
use crate::ui;

pub struct App {
    pub config: Config,
    pub client: CarpClient,
    pub cache: Cache,
    /// Signed-in account, shown in the header.
    pub account: Option<String>,

    pub route: Route,
    pub studies: StudiesState,
    pub study: Option<StudyState>,
    pub participant: Option<ParticipantState>,

    /// Where a study can be opened in a browser.
    pub portal: Portal,

    pub downloads: DownloadManager,
    pub downloads_table: ratatui::widgets::TableState,
    pub history: Vec<DownloadRecord>,

    /// The protocol editor, once it has been opened.
    pub studio: Option<crate::studio::Studio>,
    /// The upstream catalogue, held here until the editor opens.
    pub catalog: carp_catalog::Catalog,
    /// The documents it was derived from, for the templates.
    pub catalog_snapshot: Option<carp_catalog::Snapshot>,
    /// A newer upstream commit, noticed before the editor opened.
    pub catalog_update: Option<carp_catalog::Commit>,

    pub prompt: Option<Prompt>,
    pub status: Option<Status>,
    pub show_help: bool,
    pub should_quit: bool,
    /// Advances on every tick; drives the activity indicator.
    pub ticks: usize,

    tx: UnboundedSender<Message>,
    rx: UnboundedReceiver<Message>,
}

pub mod navigation;
pub mod refreshing;
pub mod transfers;

impl App {
    pub fn new(config: Config, client: CarpClient, cache: Cache, account: Option<String>) -> Self {
        let (tx, rx) = unbounded_channel();
        let portal = Portal::new(&config);
        Self {
            portal,
            config,
            client,
            cache,
            account,
            route: Route::Studies,
            studies: StudiesState::default(),
            study: None,
            participant: None,
            downloads: DownloadManager::default(),
            downloads_table: ratatui::widgets::TableState::default(),
            history: Vec::new(),
            studio: None,
            catalog: carp_catalog::Catalog::default(),
            catalog_snapshot: None,
            catalog_update: None,
            prompt: None,
            status: None,
            show_help: false,
            should_quit: false,
            ticks: 0,
            tx,
            rx,
        }
    }

    /// Draw, wait for a message, apply it. Messages that arrive together are
    /// applied in one batch so a burst of progress updates costs one redraw.
    pub async fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        tui::spawn_event_loop(self.tx.clone());

        tasks::load_cached_studies(self.cache.clone(), self.tx.clone());
        tasks::discover_portal(self.client.clone(), self.tx.clone());
        tasks::load_download_history(self.cache.clone(), self.tx.clone());
        // The catalogue is read from disk at startup so the editor opens with
        // suggestions, and upstream is asked once whether it has moved.
        studio_tasks::load_catalog(self.config.data_dir.clone(), self.tx.clone());
        studio_tasks::check_for_updates(self.config.data_dir.clone(), self.tx.clone());
        self.refresh_studies();

        while !self.should_quit {
            terminal.draw(|frame| ui::draw(frame, &mut self))?;

            let Some(message) = self.rx.recv().await else {
                break;
            };
            self.handle(message);
            while let Ok(message) = self.rx.try_recv() {
                self.handle(message);
            }
        }
        Ok(())
    }

    pub fn sender(&self) -> UnboundedSender<Message> {
        self.tx.clone()
    }

    fn handle(&mut self, message: Message) {
        match message {
            Message::Tick => {
                self.ticks = self.ticks.wrapping_add(1);
                if self.status.as_ref().is_some_and(Status::is_expired) {
                    self.status = None;
                }
                // Roughly every five seconds.
                if self.ticks.is_multiple_of(25) {
                    self.poll_pending_exports();
                }
            }
            Message::Key(key) => input::handle_key(self, key),
            Message::Redraw => {}

            Message::CachedStudies(studies) => {
                // Never overwrite a live list with a cached one.
                if self.studies.items.is_empty() {
                    self.studies.set_items(studies, true);
                }
            }
            Message::Studies(studies) => {
                self.studies.loading = false;
                self.studies.set_items(studies, false);
            }
            Message::Staff {
                study_id,
                researchers,
                assistants,
            } => {
                if let Some(study) = self.study_matching(&study_id) {
                    study.researchers = researchers;
                    study.assistants = assistants;
                    study.details_loading = false;
                    study.details_loaded = true;
                    let staff_count = study.staff().len();
                    state::clamp_selection(&mut study.staff_table, staff_count);
                }
            }
            Message::Groups { study_id, status } => {
                if let Some(study) = self.study_matching(&study_id) {
                    study.set_groups(status);
                }
            }
            Message::CachedParticipants {
                study_id,
                participants,
            } => {
                if let Some(study) = self.study_matching(&study_id)
                    && study.participants.items.is_empty()
                {
                    let total = participants.len() as u32;
                    study.participants.set_items(participants, total, true);
                }
            }
            Message::Participants { study_id, page } => {
                if let Some(study) = self.study_matching(&study_id) {
                    study.participants.loading = false;
                    let total = page.total;
                    study.participants.set_items(page.content, total, false);
                }
            }
            Message::Files { study_id, files } => {
                if let Some(study) = self.study_matching(&study_id) {
                    study.files = files;
                    study.files_loading = false;
                    study.files_loaded = true;
                    state::clamp_selection(&mut study.files_table, study.files.len());
                }
            }
            Message::Exports { study_id, exports } => {
                if let Some(study) = self.study_matching(&study_id) {
                    study.exports = exports;
                    study.exports_loading = false;
                    study.exports_loaded = true;
                    state::clamp_selection(&mut study.exports_table, study.exports.len());
                }
            }
            Message::PortalDiscovered(base) => self.portal.discovered(base),
            Message::DownloadHistory(records) => self.history = records,

            Message::DownloadProgress {
                job_id,
                received,
                total,
            } => self.downloads.progress(job_id, received, total),
            Message::DownloadFinished {
                job_id,
                path,
                bytes,
            } => {
                let job = self
                    .downloads
                    .jobs()
                    .iter()
                    .find(|job| job.id == job_id)
                    .map(|job| (job.label.clone(), job.study_id.clone()));
                let (label, study_id) = job.unwrap_or_default();
                self.downloads.finish(job_id, path.clone(), bytes);
                tasks::record_download(self.cache.clone(), study_id, label, path.clone(), bytes);
                tasks::load_download_history(self.cache.clone(), self.tx.clone());
                self.status = Some(Status::info(format!("saved {}", path.display())));
            }
            Message::DownloadFailed { job_id, error } => {
                self.downloads.fail(job_id, error.clone());
                self.status = Some(Status::error(format!("download failed: {error}")));
            }

            Message::LoadFailed {
                study_id,
                target,
                error,
            } => {
                self.clear_loading(study_id.as_deref(), target);
                self.status = Some(Status::error(error));
            }

            Message::CatalogLoaded { catalog, snapshot } => {
                self.set_catalog(*catalog, snapshot.map(|snapshot| *snapshot));
            }
            Message::CatalogMissing => {}
            Message::CatalogFailed(error) => self.set_catalog_failed(error),
            Message::CatalogUpdateAvailable(commit) => self.set_catalog_update(*commit),

            Message::ProtocolSaved(path) => self.protocol_saved(path),
            Message::ProtocolOpened { protocol, path } => self.protocol_opened(*protocol, path),
            Message::ProtocolUploaded { message, stored } => {
                self.protocol_uploaded(message, stored);
            }

            Message::Notice(text) => self.status = Some(Status::info(text)),
            Message::Error(text) => self.status = Some(Status::error(text)),
        }
    }

    /// Stop the spinner a failed load left running. What was already on
    /// screen stays there: a failed refresh must not blank the view.
    fn clear_loading(&mut self, study_id: Option<&str>, target: LoadTarget) {
        if target == LoadTarget::Studies {
            self.studies.loading = false;
            return;
        }
        let Some(study_id) = study_id else {
            return;
        };
        let Some(study) = self.study_matching(study_id) else {
            return;
        };
        match target {
            LoadTarget::Studies => {}
            LoadTarget::Details => study.details_loading = false,
            LoadTarget::Participants => study.participants.loading = false,
            LoadTarget::Files => study.files_loading = false,
            LoadTarget::Exports => study.exports_loading = false,
        }
    }

    /// The open study, when the message refers to it.
    fn study_matching(&mut self, study_id: &str) -> Option<&mut StudyState> {
        self.study
            .as_mut()
            .filter(|study| study.study.study_id.as_str() == study_id)
    }
}
