// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Reading and writing the fields every task variant shares.

use super::{AppTaskCore, KnownTask, TaskCore, TaskKind};

impl KnownTask {
    pub fn core(&self) -> &TaskCore {
        match self {
            Self::Background { core }
            | Self::Monitoring { core }
            | Self::App { core, .. }
            | Self::RpApp { core, .. }
            | Self::HealthApp { core, .. }
            | Self::Web { core, .. } => core,
        }
    }

    pub fn core_mut(&mut self) -> &mut TaskCore {
        match self {
            Self::Background { core }
            | Self::Monitoring { core }
            | Self::App { core, .. }
            | Self::RpApp { core, .. }
            | Self::HealthApp { core, .. }
            | Self::Web { core, .. } => core,
        }
    }

    pub fn app(&self) -> Option<&AppTaskCore> {
        match self {
            Self::App { app, .. } | Self::RpApp { app, .. } | Self::HealthApp { app, .. } => {
                Some(app)
            }
            _ => None,
        }
    }

    pub fn app_mut(&mut self) -> Option<&mut AppTaskCore> {
        match self {
            Self::App { app, .. } | Self::RpApp { app, .. } | Self::HealthApp { app, .. } => {
                Some(app)
            }
            _ => None,
        }
    }

    pub fn kind(&self) -> TaskKind {
        match self {
            Self::Background { .. } => TaskKind::Background,
            Self::Monitoring { .. } => TaskKind::Monitoring,
            Self::App { .. } => TaskKind::App,
            Self::RpApp { .. } => TaskKind::RpApp,
            Self::HealthApp { .. } => TaskKind::HealthApp,
            Self::Web { .. } => TaskKind::Web,
        }
    }
}
