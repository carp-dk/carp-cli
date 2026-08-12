// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! CARP CLI: a terminal client for the Copenhagen Research Platform.
//!
//! Layout of the crate:
//!
//! - [`cli`] / [`config`] - arguments, environment and local paths
//! - [`auth`] - OAuth2 session against the CARP Keycloak realm
//! - [`api`] - HTTP client, typed models, one function per API operation
//! - [`db`] - local cache of studies, participants and downloads
//! - [`download`] - streaming transfers with progress
//! - [`app`] - state, input handling and background tasks
//! - [`ui`] - rendering, one module per screen
//! - [`tui`] - terminal event loop

mod api;
mod app;
mod auth;
mod cli;
mod config;
mod db;
mod download;
mod portal;
mod protocol_cli;
mod studio;
mod tui;
mod ui;

use std::sync::Arc;

use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = cli::Args::parse()?;
    match args.command {
        cli::Command::Help => {
            println!("{}", cli::HELP);
            return Ok(());
        }
        cli::Command::Version => {
            println!("carp {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        _ => {}
    }

    let config = config::Config::load(&args)?;

    // The protocol commands work on local files and the public upstream
    // repository, so they run before authentication: checking a protocol in CI
    // must not need a CARP session.
    match &args.command {
        cli::Command::ProtocolSync => return protocol_cli::sync(&config).await,
        cli::Command::ProtocolCatalog => return protocol_cli::catalog_status(&config).await,
        cli::Command::ProtocolCheck { path } => return protocol_cli::check(path),
        cli::Command::ProtocolShow { path } => return protocol_cli::show(path),
        _ => {}
    }

    let authenticator = Arc::new(auth::Authenticator::new(&config)?);

    match &args.command {
        cli::Command::Logout => {
            authenticator.logout().await?;
            println!(
                "signed out of {}",
                config.server.as_str().trim_end_matches('/')
            );
            return Ok(());
        }
        cli::Command::Login => {
            authenticator.login().await?;
            let account = authenticator.account_label().await.unwrap_or_default();
            println!(
                "signed in to {} as {account}",
                config.server.as_str().trim_end_matches('/')
            );
            return Ok(());
        }
        _ => {}
    }

    // Log in before taking over the terminal: the flow prints a URL and waits
    // for the browser.
    authenticator.ensure_session().await?;
    let account = authenticator.account_label().await;

    let client = api::CarpClient::new(&config, authenticator)?;
    let cache = match db::Cache::open(&config.db_path()).await {
        Ok(cache) => cache,
        Err(error) => {
            // The cache is an optimisation; losing it must not stop the app.
            eprintln!("warning: local cache disabled ({error})");
            db::Cache::disabled()
        }
    };

    ui::icons::use_set(config.icons);

    // `carp protocol [path]` starts in the editor rather than the study list.
    let opening = match &args.command {
        cli::Command::Protocol { path } => Some(path.clone()),
        _ => None,
    };

    let mut app = app::App::new(config, client, cache, account);
    if let Some(path) = opening {
        app.open_studio();
        if let Some(path) = path {
            app.open_protocol_at(path);
        }
    }

    let mut terminal = ratatui::init();
    let result = app.run(&mut terminal).await;
    ratatui::restore();
    result
}
