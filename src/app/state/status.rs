// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! The transient line the status bar shows.

use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusKind {
    Info,
    Error,
}

/// Transient line shown in the status bar.
#[derive(Debug, Clone)]
pub struct Status {
    pub kind: StatusKind,
    pub text: String,
    pub raised_at: Instant,
}

impl Status {
    pub fn info(text: impl Into<String>) -> Self {
        Self {
            kind: StatusKind::Info,
            text: text.into(),
            raised_at: Instant::now(),
        }
    }

    pub fn error(text: impl Into<String>) -> Self {
        Self {
            kind: StatusKind::Error,
            text: text.into(),
            raised_at: Instant::now(),
        }
    }

    /// Errors stay until replaced; notices fade.
    pub fn is_expired(&self) -> bool {
        self.kind == StatusKind::Info && self.raised_at.elapsed().as_secs() >= 6
    }
}
