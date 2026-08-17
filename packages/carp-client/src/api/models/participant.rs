// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Participant models (`POST /api/studies/{study-id}/participants/accounts`).

use serde::{Deserialize, Serialize};

use crate::api::models::common::CarpInstant;

pub const DEFAULT_PAGE_SIZE: u32 = 50;

/// Sort keys the participant query accepts.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ParticipantSortBy {
    #[default]
    AccountIdentity,
    IsDeployed,
}

impl ParticipantSortBy {
    pub fn label(self) -> &'static str {
        match self {
            Self::AccountIdentity => "identity",
            Self::IsDeployed => "deployed",
        }
    }

    pub fn toggled(self) -> Self {
        match self {
            Self::AccountIdentity => Self::IsDeployed,
            Self::IsDeployed => Self::AccountIdentity,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    #[default]
    Asc,
    Desc,
}

impl SortDirection {
    pub fn label(self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }

    pub fn toggled(self) -> Self {
        match self {
            Self::Asc => Self::Desc,
            Self::Desc => Self::Asc,
        }
    }
}

/// Request body for the paged participant query.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticipantQuery {
    pub page: u32,
    pub size: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    pub sort_by: ParticipantSortBy,
    pub sort_direction: SortDirection,
    /// Restrict to deployed (or not yet deployed) participants. Omitted when
    /// `None`, which returns every participant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployed: Option<bool>,
}

impl Default for ParticipantQuery {
    fn default() -> Self {
        Self {
            page: 0,
            size: DEFAULT_PAGE_SIZE,
            search: None,
            sort_by: ParticipantSortBy::default(),
            sort_direction: SortDirection::default(),
            deployed: None,
        }
    }
}

/// A page of participants.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ParticipantPage {
    pub page: Option<u32>,
    pub size: Option<u32>,
    pub total: u32,
    pub content: Vec<ParticipantSummary>,
}

impl ParticipantPage {
    /// Number of pages for the given page size (at least one).
    pub fn page_count(&self, size: u32) -> u32 {
        let size = size.max(1);
        self.total.div_ceil(size).max(1)
    }
}

/// A participant-centred view of a study account.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ParticipantSummary {
    pub participant_id: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    /// Email address or username the participant was invited with.
    pub account_identity: Option<String>,
    pub invited_on: Option<CarpInstant>,
    pub carp_user: bool,
    pub deployed: bool,
}

impl ParticipantSummary {
    pub fn display_name(&self) -> String {
        let joined = [self.first_name.as_deref(), self.last_name.as_deref()]
            .into_iter()
            .flatten()
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        if joined.is_empty() {
            self.identity().to_owned()
        } else {
            joined
        }
    }

    pub fn identity(&self) -> &str {
        self.account_identity
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("-")
    }

    pub fn short_id(&self) -> &str {
        self.participant_id
            .split('-')
            .next()
            .unwrap_or(&self.participant_id)
    }

    pub fn deployment_label(&self) -> &'static str {
        if self.deployed { "deployed" } else { "invited" }
    }

    pub fn account_label(&self) -> &'static str {
        if self.carp_user {
            "CARP user"
        } else {
            "anonymous"
        }
    }
}
