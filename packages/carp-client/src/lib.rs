// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! A client for the Copenhagen Research Platform web service.
//!
//! Everything needed to talk to CARP and nothing that draws a terminal: which
//! deployment to address ([`config`]), how to hold a session against it
//! ([`auth`]), the operations themselves ([`api`]), and streaming a download to
//! disk ([`transfer`]).
//!
//! The split exists so the same code serves more than one front end. The `carp`
//! command line is one; the Python extension module is another, and a binary
//! crate could not have been linked against.
//!
//! # Layout
//!
//! - [`config`] - deployments, local paths, and the settings that pick them
//! - [`auth`] - OAuth2 session against the CARP Keycloak realm, refreshed as needed
//! - [`api`] - HTTP transport, typed payloads, one function per documented operation
//! - [`transfer`] - streaming a file or export to disk, reporting progress
//! - [`time`] - a window given as someone would type it: `7d`, `2026-08-01`
//! - [`fixtures`] - sample payloads, for tests written against real responses
//!
//! # Example
//!
//! ```no_run
//! # async fn example() -> Result<(), carp_client::Error> {
//! use std::sync::Arc;
//!
//! use carp_client::api::endpoints::studies;
//! use carp_client::{Authenticator, CarpClient, Config, Environment};
//!
//! let config = Config::for_environment(Environment::Test)?;
//! let auth = Arc::new(Authenticator::new(&config)?);
//! auth.ensure_session(|url| println!("Opening {url}")).await?;
//!
//! let client = CarpClient::new(&config, auth)?;
//! for study in studies::list(&client).await? {
//!     println!("{}", study.name);
//! }
//! # Ok(())
//! # }
//! ```

pub mod api;
pub mod auth;
pub mod config;
pub mod error;
pub mod fixtures;
pub mod time;
pub mod transfer;

pub use api::client::CarpClient;
pub use api::error::{ApiError, ApiResult};
pub use auth::Authenticator;
pub use config::{Config, Environment, Settings};
pub use error::{Error, Result};
pub use time::Moment;
