// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! What each command actually does. One module per noun.
//!
//! Everything here is thin: `carp-client` performs the operation and
//! [`crate::output`] prints it, so a command module chooses the endpoint, the
//! columns, and nothing else.

pub mod auth;
pub mod data;
pub mod deployments;
pub mod exports;
pub mod files;
pub mod participants;
pub mod protocol;
pub mod studies;

use std::process::ExitCode;
use std::sync::Arc;

use carp_client::api::CarpClient;
use carp_client::config::Config;
use carp_client::{ApiError, Authenticator, Error};
use color_eyre::Result;

use crate::cli::Global;
use crate::output::Format;

/// A connection to CARP, and how to print what comes back.
pub struct Session {
    pub client: CarpClient,
    pub config: Config,
    pub format: Format,
}

/// Open a session for a command that needs one.
///
/// Deliberately does *not* sign in when there is no session. A command may be
/// running in a pipe, a cron job or a notebook, and opening a browser there is
/// at best a surprise and at worst a hang nobody can see. Signing in is
/// something you ask for, by name: `carp auth login`.
pub async fn connect(global: &Global) -> Result<Session> {
    let config = Config::load(&global.settings())?;
    let authenticator = Arc::new(Authenticator::new(&config)?);

    if !authenticator.has_session().await {
        return Err(Error::no_session(format!(
            "not signed in to {} - run `carp auth login`",
            server_label(&config)
        ))
        .into());
    }

    let client = CarpClient::new(&config, authenticator)?;
    Ok(Session {
        client,
        config,
        format: global.format(),
    })
}

/// The server a message should name, without its trailing slash.
pub fn server_label(config: &Config) -> &str {
    config.server.as_str().trim_end_matches('/')
}

/// What the process exits with.
///
/// A script should be able to tell "you are not signed in" from "that study
/// does not exist" from "the network is down" without reading the message, so
/// the ones worth branching on get their own code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Exit {
    Ok = 0,
    /// Anything not covered below.
    Failure = 1,
    /// The arguments did not parse. Chosen by clap, matched here for the docs.
    Usage = 2,
    /// No session, or one the server rejected.
    NoSession = 3,
    NotFound = 4,
    Forbidden = 5,
}

impl From<Exit> for ExitCode {
    fn from(exit: Exit) -> Self {
        Self::from(exit as u8)
    }
}

impl Exit {
    /// Classify a failure by what the caller would do about it.
    pub fn of(error: &color_eyre::Report) -> Self {
        match error.downcast_ref::<Error>() {
            Some(Error::Api(api)) => Self::of_api(api),
            Some(Error::NoSession(_) | Error::Login(_)) => Self::NoSession,
            Some(Error::Config(_) | Error::Io { .. }) | None => {
                // An ApiError may also arrive unwrapped, from a call that
                // returned one directly rather than through carp_client::Error.
                error
                    .downcast_ref::<ApiError>()
                    .map_or(Self::Failure, Self::of_api)
            }
        }
    }

    fn of_api(error: &ApiError) -> Self {
        match error {
            ApiError::Unauthorized(_) => Self::NoSession,
            ApiError::Forbidden(_) => Self::Forbidden,
            ApiError::NotFound(_) => Self::NotFound,
            _ => Self::Failure,
        }
    }

    /// The name a JSON error report carries, so a caller can branch on the
    /// word rather than on the number.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Failure => "failure",
            Self::Usage => "usage",
            Self::NoSession => "no-session",
            Self::NotFound => "not-found",
            Self::Forbidden => "forbidden",
        }
    }
}

#[cfg(test)]
mod tests;
