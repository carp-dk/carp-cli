// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! API errors, shaped so the TUI can show a single useful line.

use std::fmt;

use serde::Deserialize;

pub type ApiResult<T> = std::result::Result<T, ApiError>;

/// Error body returned by CARP (`CarpErrorResponse`).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CarpErrorResponse {
    pub status_code: i32,
    pub exception: String,
    pub message: String,
    pub path: String,
}

#[derive(Debug)]
pub enum ApiError {
    /// No valid session, or the server rejected the one we have.
    Unauthorized(String),
    /// The caller is signed in but not allowed to see this resource.
    Forbidden(String),
    NotFound(String),
    /// Any other non-success status.
    Status {
        status: u16,
        message: String,
    },
    /// Connection level failure.
    Transport(String),
    /// The response body did not match what the client expects.
    Decode(String),
    /// Writing a download to disk failed.
    Io(String),
}
impl ApiError {
    pub fn from_status(status: u16, body: &str) -> Self {
        let message = serde_json::from_str::<CarpErrorResponse>(body)
            .ok()
            .map(|error| {
                if error.message.is_empty() {
                    error.exception
                } else {
                    error.message
                }
            })
            .filter(|message| !message.is_empty())
            .unwrap_or_else(|| {
                let body = body.trim();
                if body.is_empty() {
                    format!("HTTP {status}")
                } else {
                    body.chars().take(200).collect()
                }
            });

        match status {
            401 => Self::Unauthorized(message),
            403 => Self::Forbidden(message),
            404 => Self::NotFound(message),
            _ => Self::Status { status, message },
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unauthorized(message) => write!(f, "not authorised: {message}"),
            Self::Forbidden(message) => write!(f, "forbidden: {message}"),
            Self::NotFound(message) => write!(f, "not found: {message}"),
            Self::Status { status, message } => write!(f, "HTTP {status}: {message}"),
            Self::Transport(message) => write!(f, "connection failed: {message}"),
            Self::Decode(message) => write!(f, "unexpected response: {message}"),
            Self::Io(message) => write!(f, "write failed: {message}"),
        }
    }
}

impl std::error::Error for ApiError {}

impl From<reqwest::Error> for ApiError {
    fn from(error: reqwest::Error) -> Self {
        if error.is_decode() {
            Self::Decode(error.to_string())
        } else {
            Self::Transport(error.to_string())
        }
    }
}

impl From<std::io::Error> for ApiError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}
