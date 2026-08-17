// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Reading and writing the fields every trigger variant shares.
//!
//! Mechanical dispatch, kept apart from the type definitions so the shape of
//! the schema stays readable in [`super`].

use super::{KnownTrigger, TriggerKind};

impl KnownTrigger {
    /// The device this trigger is evaluated on.
    pub fn source_device(&self) -> &str {
        match self {
            Self::NoOp {
                source_device_role_name,
            }
            | Self::Immediate {
                source_device_role_name,
            }
            | Self::OneTime {
                source_device_role_name,
            }
            | Self::Periodic {
                source_device_role_name,
                ..
            }
            | Self::RecurrentScheduled {
                source_device_role_name,
                ..
            }
            | Self::CronScheduled {
                source_device_role_name,
                ..
            }
            | Self::UserTask {
                source_device_role_name,
                ..
            }
            | Self::NoUserTask {
                source_device_role_name,
                ..
            }
            | Self::SamplingEvent {
                source_device_role_name,
                ..
            } => source_device_role_name,
        }
    }

    pub(super) fn set_source_device(&mut self, device: String) {
        match self {
            Self::NoOp {
                source_device_role_name,
            }
            | Self::Immediate {
                source_device_role_name,
            }
            | Self::OneTime {
                source_device_role_name,
            }
            | Self::Periodic {
                source_device_role_name,
                ..
            }
            | Self::RecurrentScheduled {
                source_device_role_name,
                ..
            }
            | Self::CronScheduled {
                source_device_role_name,
                ..
            }
            | Self::UserTask {
                source_device_role_name,
                ..
            }
            | Self::NoUserTask {
                source_device_role_name,
                ..
            }
            | Self::SamplingEvent {
                source_device_role_name,
                ..
            } => *source_device_role_name = device,
        }
    }

    pub fn kind(&self) -> TriggerKind {
        match self {
            Self::NoOp { .. } => TriggerKind::NoOp,
            Self::Immediate { .. } => TriggerKind::Immediate,
            Self::OneTime { .. } => TriggerKind::OneTime,
            Self::Periodic { .. } => TriggerKind::Periodic,
            Self::RecurrentScheduled { .. } => TriggerKind::RecurrentScheduled,
            Self::CronScheduled { .. } => TriggerKind::CronScheduled,
            Self::UserTask { .. } => TriggerKind::UserTask,
            Self::NoUserTask { .. } => TriggerKind::NoUserTask,
            Self::SamplingEvent { .. } => TriggerKind::SamplingEvent,
        }
    }
}
