// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Local cache of the last data seen for each study.
//!
//! The cache exists so the UI has something to show immediately at startup and
//! keeps working when the network does not. It is never authoritative: a
//! successful API response always replaces it, and a cache failure is reported
//! but never fatal.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::Utc;
use color_eyre::Result;
use color_eyre::eyre::Context;
use turso::{Builder, Connection, Database, Value};

use crate::db::schema::MIGRATIONS;
use carp_client::api::models::{ParticipantSummary, StudyOverview};

/// A completed download, as recorded in the cache.
#[derive(Debug, Clone)]
pub struct DownloadRecord {
    pub label: String,
    pub path: PathBuf,
    pub bytes: u64,
    pub finished_at: String,
}

#[derive(Clone)]
pub struct Cache {
    inner: Option<Arc<Inner>>,
}

struct Inner {
    #[allow(dead_code, reason = "kept alive so the connection stays valid")]
    database: Database,
    connection: Connection,
}

impl Cache {
    /// Open (creating if needed) the cache database and apply the schema.
    pub async fn open(path: &Path) -> Result<Self> {
        let database = Builder::new_local(&path.to_string_lossy())
            .build()
            .await
            .with_context(|| format!("opening cache database {}", path.display()))?;
        let connection = database.connect().context("connecting to the cache")?;
        connection
            .execute_batch(MIGRATIONS)
            .await
            .context("applying the cache schema")?;

        Ok(Self {
            inner: Some(Arc::new(Inner {
                database,
                connection,
            })),
        })
    }

    /// A cache that silently drops everything, used when the database could
    /// not be opened.
    pub fn disabled() -> Self {
        Self { inner: None }
    }

    fn connection(&self) -> Option<&Connection> {
        self.inner.as_ref().map(|inner| &inner.connection)
    }

    /// Replace the cached study list.
    pub async fn save_studies(&self, studies: &[StudyOverview]) -> Result<()> {
        let Some(connection) = self.connection() else {
            return Ok(());
        };
        let now = Utc::now().to_rfc3339();
        connection.execute("DELETE FROM studies", ()).await?;
        for study in studies {
            let payload = serde_json::to_string(study)?;
            connection
                .execute(
                    "INSERT INTO studies (study_id, name, payload, cached_at) VALUES (?, ?, ?, ?)",
                    vec![
                        Value::Text(study.study_id.to_string()),
                        Value::Text(study.name.clone()),
                        Value::Text(payload),
                        Value::Text(now.clone()),
                    ],
                )
                .await?;
        }
        Ok(())
    }

    /// Studies from the last successful refresh.
    pub async fn load_studies(&self) -> Result<Vec<StudyOverview>> {
        let Some(connection) = self.connection() else {
            return Ok(Vec::new());
        };
        let mut rows = connection
            .query("SELECT payload FROM studies ORDER BY name", ())
            .await?;
        let mut studies = Vec::new();
        while let Some(row) = rows.next().await? {
            if let Some(payload) = text(&row.get_value(0)?)
                && let Ok(study) = serde_json::from_str(&payload)
            {
                studies.push(study);
            }
        }
        Ok(studies)
    }

    /// Replace the cached participants of one study.
    pub async fn save_participants(
        &self,
        study_id: &str,
        participants: &[ParticipantSummary],
    ) -> Result<()> {
        let Some(connection) = self.connection() else {
            return Ok(());
        };
        let now = Utc::now().to_rfc3339();
        connection
            .execute(
                "DELETE FROM participants WHERE study_id = ?",
                vec![Value::Text(study_id.to_owned())],
            )
            .await?;
        for participant in participants {
            let payload = serde_json::to_string(participant)?;
            connection
                .execute(
                    "INSERT INTO participants (id, study_id, payload, cached_at) \
                     VALUES (?, ?, ?, ?)",
                    vec![
                        Value::Text(format!("{study_id}:{}", participant.participant_id)),
                        Value::Text(study_id.to_owned()),
                        Value::Text(payload),
                        Value::Text(now.clone()),
                    ],
                )
                .await?;
        }
        Ok(())
    }

    /// Participants of a study from the last successful refresh.
    pub async fn load_participants(&self, study_id: &str) -> Result<Vec<ParticipantSummary>> {
        let Some(connection) = self.connection() else {
            return Ok(Vec::new());
        };
        let mut rows = connection
            .query(
                "SELECT payload FROM participants WHERE study_id = ? ORDER BY id",
                vec![Value::Text(study_id.to_owned())],
            )
            .await?;
        let mut participants = Vec::new();
        while let Some(row) = rows.next().await? {
            if let Some(payload) = text(&row.get_value(0)?)
                && let Ok(participant) = serde_json::from_str(&payload)
            {
                participants.push(participant);
            }
        }
        Ok(participants)
    }

    /// Remember where a download ended up.
    pub async fn record_download(
        &self,
        study_id: Option<&str>,
        label: &str,
        path: &Path,
        bytes: u64,
    ) -> Result<()> {
        let Some(connection) = self.connection() else {
            return Ok(());
        };
        connection
            .execute(
                "INSERT INTO downloads (study_id, label, path, bytes, finished_at) \
                 VALUES (?, ?, ?, ?, ?)",
                vec![
                    study_id.map_or(Value::Null, |id| Value::Text(id.to_owned())),
                    Value::Text(label.to_owned()),
                    Value::Text(path.to_string_lossy().into_owned()),
                    Value::Integer(bytes as i64),
                    Value::Text(Utc::now().to_rfc3339()),
                ],
            )
            .await?;
        Ok(())
    }

    /// Most recent downloads, newest first.
    pub async fn recent_downloads(&self, limit: u32) -> Result<Vec<DownloadRecord>> {
        let Some(connection) = self.connection() else {
            return Ok(Vec::new());
        };
        let mut rows = connection
            .query(
                "SELECT label, path, bytes, finished_at FROM downloads \
                 ORDER BY id DESC LIMIT ?",
                vec![Value::Integer(i64::from(limit))],
            )
            .await?;
        let mut records = Vec::new();
        while let Some(row) = rows.next().await? {
            let label = text(&row.get_value(0)?).unwrap_or_default();
            let path = text(&row.get_value(1)?).unwrap_or_default();
            let bytes = integer(&row.get_value(2)?).unwrap_or(0);
            let finished_at = text(&row.get_value(3)?).unwrap_or_default();
            records.push(DownloadRecord {
                label,
                path: PathBuf::from(path),
                bytes: bytes.max(0) as u64,
                finished_at,
            });
        }
        Ok(records)
    }
}

fn text(value: &Value) -> Option<String> {
    match value {
        Value::Text(text) => Some(text.clone()),
        _ => None,
    }
}

fn integer(value: &Value) -> Option<i64> {
    match value {
        Value::Integer(number) => Some(*number),
        _ => None,
    }
}
