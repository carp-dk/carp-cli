// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! `carp studies` - what you have access to, and one of them in detail.

use carp_client::ApiError;
use carp_client::api::endpoints::studies;
use carp_client::api::models::{Account, ParticipantGroupStatus, StudyOverview, format_instant};
use color_eyre::Result;
use serde::Serialize;

use crate::cli::{Global, StudiesCommand};
use crate::commands::{Session, connect};
use crate::output::{self, Rows};

pub async fn run(command: Option<&StudiesCommand>, global: &Global) -> Result<()> {
    let session = connect(global).await?;
    match command {
        // Bare `carp studies` lists them, which is what it would have to mean.
        None | Some(StudiesCommand::List { search: None }) => list(&session, None).await,
        Some(StudiesCommand::List { search }) => list(&session, search.as_deref()).await,
        Some(StudiesCommand::Show { study }) => show(&session, study).await,
    }
}

impl Rows for StudyOverview {
    const HEADERS: &'static [&'static str] = &["id", "name", "stage", "created", "owner"];

    fn cells(&self) -> Vec<String> {
        vec![
            self.study_id.to_string(),
            self.name.clone(),
            self.stage().to_owned(),
            format_instant(self.created_on),
            self.created_by.clone().unwrap_or_else(|| "-".to_owned()),
        ]
    }
}

async fn list(session: &Session, search: Option<&str>) -> Result<()> {
    let mut studies = studies::list(&session.client).await?;
    if let Some(needle) = search {
        studies.retain(|study| study.matches(needle));
    }
    output::rows(&studies, session.format)
}

/// A study with everything a listing leaves out.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StudyDetail {
    #[serde(flatten)]
    study: StudyOverview,
    researchers: Vec<Account>,
    research_assistants: Vec<Account>,
    groups: ParticipantGroupStatus,
}

async fn show(session: &Session, study_id: &str) -> Result<()> {
    let study = studies::list(&session.client)
        .await?
        .into_iter()
        .find(|study| study.study_id.as_str() == study_id)
        .ok_or_else(|| {
            // The overview is the only listing of studies, so "not in it" is
            // the whole of what can be known — say which server was asked.
            ApiError::NotFound(format!(
                "no study {study_id} on {}",
                crate::commands::server_label(&session.config)
            ))
        })?;

    // Staff and groups are separate calls; a study with neither is still a
    // study, so a failure on one must not lose the rest.
    let (researchers, assistants, groups) = tokio::join!(
        studies::researchers(&session.client, study_id),
        studies::research_assistants(&session.client, study_id),
        studies::participant_group_status(&session.client, study_id),
    );

    let detail = StudyDetail {
        researchers: staff(researchers, "researchers")?,
        research_assistants: staff(assistants, "research assistants")?,
        groups: groups.unwrap_or_default(),
        study,
    };

    let lines = vec![
        ("id", detail.study.study_id.to_string()),
        ("name", detail.study.name.clone()),
        ("stage", detail.study.stage().to_owned()),
        ("about", detail.study.description_line().to_owned()),
        ("created", format_instant(detail.study.created_on)),
        (
            "owner",
            detail
                .study
                .created_by
                .clone()
                .unwrap_or_else(|| "-".to_owned()),
        ),
        (
            "protocol",
            detail
                .study
                .study_protocol_id
                .as_ref()
                .map_or_else(|| "-".to_owned(), ToString::to_string),
        ),
        ("researchers", names(&detail.researchers)),
        ("assistants", names(&detail.research_assistants)),
        ("groups", detail.groups.summary()),
    ];
    output::detail(&detail, &lines, session.format)
}

/// A staff list that could not be read is reported, not silently empty: an
/// empty list and an unreadable one mean very different things about a study.
fn staff(result: Result<Vec<Account>, ApiError>, what: &str) -> Result<Vec<Account>> {
    match result {
        Ok(accounts) => Ok(accounts),
        Err(ApiError::Forbidden(_)) => {
            output::note(format!("note: not allowed to see this study's {what}"));
            Ok(Vec::new())
        }
        Err(error) => Err(error.into()),
    }
}

fn names(accounts: &[Account]) -> String {
    if accounts.is_empty() {
        return "-".to_owned();
    }
    accounts
        .iter()
        .map(Account::display_name)
        .collect::<Vec<_>>()
        .join(", ")
}
