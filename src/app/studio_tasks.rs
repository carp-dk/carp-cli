// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Background work for the protocol editor.
//!
//! Everything here is spawned and answers with a [`Message`], so the editor
//! never blocks on a network request or a disk write. That matters most for
//! the catalogue sync, which fetches a dozen files and would otherwise freeze
//! the interface for a second or two.

use std::path::PathBuf;

use tokio::sync::mpsc::UnboundedSender;

use carp_protocol::StudyProtocol;

use crate::api::CarpClient;
use crate::api::endpoints::protocols;
use crate::app::message::Message;

/// Load the stored catalogue, without touching the network.
///
/// Called at startup. A missing catalogue is not an error: the editor works
/// without one, offering no suggestions until a sync is asked for.
pub fn load_catalog(data_dir: PathBuf, tx: UnboundedSender<Message>) {
    tokio::spawn(async move {
        let snapshot = carp_catalog::sync::load_snapshot(&data_dir).await;
        let message = match snapshot {
            Ok(snapshot) => {
                let catalog = carp_catalog::derive::catalog(&snapshot);
                Message::CatalogLoaded {
                    catalog: Box::new(catalog),
                    snapshot: Some(Box::new(snapshot)),
                }
            }
            Err(carp_catalog::Error::Missing) => Message::CatalogMissing,
            Err(error) => Message::CatalogFailed(error.to_string()),
        };
        let _ = tx.send(message);
    });
}

/// Download the upstream protocols and rebuild the catalogue.
pub fn sync_catalog(data_dir: PathBuf, tx: UnboundedSender<Message>) {
    tokio::spawn(async move {
        let message = match carp_catalog::sync::sync(&data_dir).await {
            Ok(report) => {
                let summary = report.summary();
                // The snapshot is re-read rather than returned by the sync,
                // because the sync's report carries the derived catalogue but
                // not the documents the templates need.
                let snapshot = carp_catalog::sync::load_snapshot(&data_dir).await.ok();
                let _ = tx.send(Message::Notice(summary));
                Message::CatalogLoaded {
                    catalog: Box::new(report.catalog),
                    snapshot: snapshot.map(Box::new),
                }
            }
            Err(error) => Message::CatalogFailed(error.to_string()),
        };
        let _ = tx.send(message);
    });
}

/// Ask whether upstream has moved, without changing anything.
///
/// Runs once at startup. The answer is shown in the Catalog tab; the
/// catalogue itself is only replaced when a sync is asked for, so a value
/// cannot vanish from a picker while it is open.
pub fn check_for_updates(data_dir: PathBuf, tx: UnboundedSender<Message>) {
    tokio::spawn(async move {
        // A failure here is silent: an offline machine should not be told
        // about its network every time the editor opens.
        if let Ok(Some(commit)) = carp_catalog::sync::check_for_updates(&data_dir).await {
            let _ = tx.send(Message::CatalogUpdateAvailable(Box::new(commit)));
        }
    });
}

/// Write a protocol to disk.
pub fn save_protocol(protocol: StudyProtocol, path: PathBuf, tx: UnboundedSender<Message>) {
    tokio::spawn(async move {
        // Blocking file I/O on the runtime's blocking pool: a protocol is
        // small, but a slow or full disk must not stall the event loop.
        let write = tokio::task::spawn_blocking({
            let path = path.clone();
            move || crate::studio::storage::write(&protocol, &path)
        })
        .await;

        let message = match write {
            Ok(Ok(())) => Message::ProtocolSaved(path),
            Ok(Err(error)) => Message::Error(format!("could not save: {error:#}")),
            Err(error) => Message::Error(format!("saving did not finish: {error}")),
        };
        let _ = tx.send(message);
    });
}

/// Read a protocol from disk.
pub fn open_protocol(path: PathBuf, tx: UnboundedSender<Message>) {
    tokio::spawn(async move {
        let read = tokio::task::spawn_blocking({
            let path = path.clone();
            move || crate::studio::storage::read_checked(&path)
        })
        .await;

        let message = match read {
            Ok(Ok((protocol, resolved))) => Message::ProtocolOpened {
                protocol: Box::new(protocol),
                path: resolved,
            },
            Ok(Err(error)) => Message::Error(format!("could not open: {error:#}")),
            Err(error) => Message::Error(format!("opening did not finish: {error}")),
        };
        let _ = tx.send(message);
    });
}

/// Upload a protocol to CAWS under `version_tag`.
pub fn upload_protocol(
    client: CarpClient,
    protocol: StudyProtocol,
    version_tag: String,
    tx: UnboundedSender<Message>,
) {
    tokio::spawn(async move {
        let message = match protocols::store(&client, &protocol, &version_tag).await {
            Ok(outcome) => Message::ProtocolUploaded {
                message: outcome.message(),
                stored: outcome.is_stored(),
            },
            Err(error) => Message::Error(format!("upload failed: {error}")),
        };
        let _ = tx.send(message);
    });
}
