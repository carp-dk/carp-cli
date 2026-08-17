// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Everything that can change the application state arrives as a [`Message`]:
//! terminal input on one side, results of background work on the other.

use std::path::PathBuf;

use ratatui::crossterm::event::KeyEvent;

use crate::db::DownloadRecord;
use crate::download::JobId;
use carp_client::api::models::{
    Account, Export, ParticipantGroupStatus, ParticipantPage, StudyFile, StudyOverview,
};

/// What a background load was fetching, so a failure clears the right
/// spinner without discarding what is already on screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadTarget {
    Studies,
    /// Staff and participant groups, which one request loads together.
    Details,
    Participants,
    Files,
    Exports,
}

#[derive(Debug)]
pub enum Message {
    /// Periodic tick, drives spinners and status expiry.
    Tick,
    Key(KeyEvent),
    /// Terminal resized; redraw is enough.
    Redraw,

    /// Studies read from the local cache at startup.
    CachedStudies(Vec<StudyOverview>),
    /// Studies from the API.
    Studies(Vec<StudyOverview>),
    /// Researchers and research assistants of a study.
    Staff {
        study_id: String,
        researchers: Vec<Account>,
        assistants: Vec<Account>,
    },
    Groups {
        study_id: String,
        status: ParticipantGroupStatus,
    },
    CachedParticipants {
        study_id: String,
        participants: Vec<carp_client::api::models::ParticipantSummary>,
    },
    Participants {
        study_id: String,
        page: ParticipantPage,
    },
    Files {
        study_id: String,
        files: Vec<StudyFile>,
    },
    Exports {
        study_id: String,
        exports: Vec<Export>,
    },
    /// Address of the web portal, learned from the server.
    PortalDiscovered(url::Url),
    /// Downloads restored from the cache log.
    DownloadHistory(Vec<DownloadRecord>),

    DownloadProgress {
        job_id: JobId,
        received: u64,
        total: Option<u64>,
    },
    DownloadFinished {
        job_id: JobId,
        path: PathBuf,
        bytes: u64,
    },
    DownloadFailed {
        job_id: JobId,
        error: String,
    },

    /// A load failed: clear its spinner, keep the previous contents.
    LoadFailed {
        study_id: Option<String>,
        target: LoadTarget,
        error: String,
    },

    /// The stored catalogue, and the documents it was derived from.
    CatalogLoaded {
        catalog: Box<carp_catalog::Catalog>,
        snapshot: Option<Box<carp_catalog::Snapshot>>,
    },
    /// No catalogue has been downloaded yet.
    CatalogMissing,
    /// Loading or syncing the catalogue failed.
    CatalogFailed(String),
    /// Upstream has moved past the stored catalogue.
    CatalogUpdateAvailable(Box<carp_catalog::Commit>),

    /// A protocol was written to disk.
    ProtocolSaved(PathBuf),
    /// A protocol was read from disk.
    ProtocolOpened {
        protocol: Box<carp_protocol::StudyProtocol>,
        path: PathBuf,
    },
    /// A protocol reached CAWS, or was refused by it.
    ProtocolUploaded {
        message: String,
        stored: bool,
    },

    /// Transient success/info line for the status bar.
    Notice(String),
    /// Transient failure line for the status bar.
    Error(String),
}
