// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! [`TaskKind`]: the task classes the editor can create.

use super::{AppTaskCore, KnownTask, Task, TaskCore};
use crate::survey::RpTask;

/// Measures every protocol should collect about itself. A protocol without
/// them still runs, but nothing reports when a task failed or was completed,
/// which makes a study impossible to debug after the fact.
pub const MONITORING_MEASURES: [&str; 3] = [
    "dk.cachet.carp.error",
    "dk.cachet.carp.triggeredtask",
    "dk.cachet.carp.completedtask",
];

/// A task class that can be added to a protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskKind {
    Background,
    Monitoring,
    App,
    RpApp,
    HealthApp,
    Web,
}

impl TaskKind {
    /// Every kind, ordered as the editor's picker shows them.
    pub const ALL: [Self; 6] = [
        Self::Background,
        Self::RpApp,
        Self::App,
        Self::HealthApp,
        Self::Web,
        Self::Monitoring,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Background => "BackgroundTask",
            Self::Monitoring => "MonitoringTask",
            Self::App => "AppTask",
            Self::RpApp => "RPAppTask",
            Self::HealthApp => "HealthAppTask",
            Self::Web => "WebTask",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Background => "Collects measures silently, with no card in the app",
            Self::Monitoring => "Collects the protocol's own errors and task events",
            Self::App => "A card the participant opens: audio, image, or a custom type",
            Self::RpApp => "A card presenting a Research Package survey",
            Self::HealthApp => "A card that reads metrics from the phone's health database",
            Self::Web => "A card that opens a web page",
        }
    }

    pub fn type_name(self) -> &'static str {
        match self {
            Self::Background => "dk.cachet.carp.common.application.tasks.BackgroundTask",
            Self::Monitoring => "dk.cachet.carp.common.application.tasks.MonitoringTask",
            Self::App => "dk.cachet.carp.common.application.tasks.AppTask",
            Self::RpApp => "dk.cachet.carp.common.application.tasks.RPAppTask",
            Self::HealthApp => "dk.cachet.carp.common.application.tasks.HealthAppTask",
            Self::Web => "dk.cachet.carp.common.application.tasks.WebTask",
        }
    }

    /// Whether the participant sees a card for tasks of this kind.
    pub fn is_app_task(self) -> bool {
        matches!(self, Self::App | Self::RpApp | Self::HealthApp | Self::Web)
    }

    /// The `type` string an app task of this kind defaults to. The study app
    /// picks the card's icon from it.
    pub fn default_app_type(self) -> &'static str {
        match self {
            Self::RpApp => "survey",
            Self::HealthApp => "health",
            _ => "other",
        }
    }

    pub fn from_type_name(type_name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.type_name() == type_name)
    }

    /// Build a task of this kind called `name`.
    ///
    /// The result is deliberately minimal - an empty survey, no measures on a
    /// background task - because the editor fills the rest in. The one
    /// exception is [`TaskKind::Monitoring`], which is only useful with its
    /// three standard measures and so ships with them.
    pub fn instantiate(self, name: String) -> Task {
        let core = TaskCore {
            name: name.clone(),
            measures: match self {
                Self::Monitoring => MONITORING_MEASURES
                    .iter()
                    .map(|data_type| super::Measure::data_stream(*data_type))
                    .collect(),
                _ => Vec::new(),
            },
        };

        let app = AppTaskCore {
            r#type: self.default_app_type().to_owned(),
            title: name.clone(),
            description: String::new(),
            instructions: String::new(),
        };

        let task = match self {
            Self::Background => KnownTask::Background { core },
            Self::Monitoring => KnownTask::Monitoring { core },
            Self::App => KnownTask::App {
                core,
                app,
                minutes_to_complete: None,
                notification: false,
            },
            Self::RpApp => KnownTask::RpApp {
                core,
                app,
                minutes_to_complete: None,
                expire: None,
                notification: false,
                rp_task: RpTask::ordered(survey_identifier(&name)),
            },
            Self::HealthApp => KnownTask::HealthApp {
                core,
                app,
                notification: false,
                types: Vec::new(),
            },
            Self::Web => KnownTask::Web {
                core,
                description: String::new(),
                url: String::new(),
            },
        };
        Task::Known(Box::new(task))
    }
}

/// Turn a task name into an identifier a survey can use: lower case, words
/// joined by underscores, nothing else. `Sleep Diary` becomes `sleep_diary`.
pub fn survey_identifier(name: &str) -> String {
    let mut identifier = String::with_capacity(name.len());
    let mut pending_separator = false;

    for character in name.chars() {
        if character.is_alphanumeric() {
            if pending_separator && !identifier.is_empty() {
                identifier.push('_');
            }
            pending_separator = false;
            identifier.extend(character.to_lowercase());
        } else {
            pending_separator = true;
        }
    }

    if identifier.is_empty() {
        "survey".to_owned()
    } else {
        identifier
    }
}

#[cfg(test)]
mod tests;
