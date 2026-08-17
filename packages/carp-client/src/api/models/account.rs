// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Account models (`account-controller`, study staff endpoints).

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountRole {
    #[default]
    Unknown,
    Participant,
    ResearchAssistant,
    Researcher,
    CarpAdmin,
    SystemAdmin,
}

impl AccountRole {
    pub fn label(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Participant => "participant",
            Self::ResearchAssistant => "research assistant",
            Self::Researcher => "researcher",
            Self::CarpAdmin => "CARP admin",
            Self::SystemAdmin => "system admin",
        }
    }
}

impl<'de> Deserialize<'de> for AccountRole {
    /// Roles added server-side must not break the client, so anything
    /// unrecognised degrades to `Unknown`.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Ok(match value.as_str() {
            "PARTICIPANT" => Self::Participant,
            "RESEARCH_ASSISTANT" => Self::ResearchAssistant,
            "RESEARCHER" => Self::Researcher,
            "CARP_ADMIN" => Self::CarpAdmin,
            "SYSTEM_ADMIN" => Self::SystemAdmin,
            _ => Self::Unknown,
        })
    }
}

impl std::fmt::Display for AccountRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// A CARP account, as returned for study researchers and assistants.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Account {
    pub id: Option<String>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub full_name: Option<String>,
    pub role: Option<AccountRole>,
}

impl Account {
    /// Best available human name for the account.
    pub fn display_name(&self) -> String {
        if let Some(name) = self
            .full_name
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            return name.to_owned();
        }
        let joined = [self.first_name.as_deref(), self.last_name.as_deref()]
            .into_iter()
            .flatten()
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        if !joined.is_empty() {
            return joined;
        }
        self.identifier().to_owned()
    }

    /// Email, falling back to username or id.
    pub fn identifier(&self) -> &str {
        self.email
            .as_deref()
            .or(self.username.as_deref())
            .or(self.id.as_deref())
            .unwrap_or("-")
    }

    pub fn role_label(&self) -> &'static str {
        self.role.unwrap_or_default().label()
    }
}
