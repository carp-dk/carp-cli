// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! CARP CLI: a client for the Copenhagen Research Platform.
//!
//! Talking to CARP is `carp-client`'s job. What lives here are the two ways of
//! asking it to: a command line, and — behind the `tui` feature — an
//! interactive browser and protocol editor.
//!
//! The command line is the one to build on. It prints JSON when its output is
//! not a terminal, exits with a code that says what went wrong, and never opens
//! a browser it was not asked to, so it works the same in a shell, a CI job and
//! a Python subprocess.
//!
//! Layout of the crate:
//!
//! - [`cli`] - the argument surface, as clap definitions
//! - [`commands`] - what each one does; one module per noun
//! - [`output`] - table, JSON, NDJSON and CSV rendering
//! - [`transfer`] - downloading, with progress when someone is watching
//! - [`protocol_file`] - where a `protocol.json` lives on disk
//! - [`app`], [`ui`], [`tui`], [`studio`], [`db`] - the interactive browser

mod cli;
mod commands;
mod output;
mod protocol_file;
mod transfer;

#[cfg(feature = "tui")]
mod app;
#[cfg(feature = "tui")]
mod db;
#[cfg(feature = "tui")]
mod download;
#[cfg(feature = "tui")]
mod portal;
#[cfg(feature = "tui")]
mod studio;
#[cfg(feature = "tui")]
mod tui;
#[cfg(feature = "tui")]
mod ui;

use std::io::Write;
use std::process::ExitCode;

use clap::{CommandFactory, Parser};
use color_eyre::Result;

use crate::cli::{Cli, Command};
use crate::commands::Exit;
use crate::output::Format;

/// Let a closed pipe end the process, the way every other Unix tool does.
///
/// Rust ignores `SIGPIPE` so that writes fail with `EPIPE` instead. That suits
/// a program that wants to notice; it does not suit this one. `carp studies
/// list | head` closes the pipe on purpose, and the reward for noticing is a
/// panic from whichever `println!` was mid-write. Restoring the default
/// disposition turns that back into what the shell expects.
#[cfg(unix)]
fn end_quietly_on_a_closed_pipe() {
    // SAFETY: called once, before the runtime starts any other thread, and
    // only to restore a disposition the platform set in the first place.
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}

#[cfg(not(unix))]
fn end_quietly_on_a_closed_pipe() {}

#[tokio::main]
async fn main() -> ExitCode {
    end_quietly_on_a_closed_pipe();

    // The colour-eyre panic hook only; its error hook would print a report
    // where a one-line message belongs. `run` reports its own failures.
    if let Err(error) = color_eyre::config::HookBuilder::default()
        .display_env_section(false)
        .install()
    {
        eprintln!("carp: {error}");
        return Exit::Failure.into();
    }

    let cli = Cli::parse();
    let format = cli.global.format();

    match run(cli).await {
        Ok(()) => Exit::Ok.into(),
        Err(error) => {
            report(&error, format);
            Exit::of(&error).into()
        }
    }
}

async fn run(cli: Cli) -> Result<()> {
    let global = &cli.global;
    match &cli.command {
        Command::Auth { command } => commands::auth::run(command, global).await,
        Command::Studies { command } => commands::studies::run(command.as_ref(), global).await,
        Command::Participants { command } => {
            commands::participants::run(command.as_ref(), global).await
        }
        Command::Deployments { command } => commands::deployments::run(command, global).await,
        Command::Data { command } => commands::data::run(command, global).await,
        Command::Export { command } => commands::exports::run(command, global).await,
        Command::Files { command } => commands::files::run(command, global).await,

        Command::Protocol { command } => {
            // Everything but `edit` is non-interactive and handled outright.
            if commands::protocol::run(command.as_ref(), global).await? {
                return Ok(());
            }
            let path = match command {
                Some(cli::ProtocolCommand::Edit { path }) => path.clone(),
                _ => None,
            };
            interactive(&cli, Some(path)).await
        }

        Command::Tui => interactive(&cli, None).await,

        Command::Completions { shell } => {
            let mut command = Cli::command();
            let name = command.get_name().to_string();
            clap_complete::generate(*shell, &mut command, name, &mut std::io::stdout());
            Ok(())
        }
    }
}

/// Report a failure to whoever is reading.
///
/// As JSON when the output is being parsed, so a caller never has to read
/// prose to find out what happened; as a sentence otherwise. Either way on
/// stderr, so a partial result already on stdout stays parseable.
fn report(error: &color_eyre::Report, format: Format) {
    let exit = Exit::of(error);
    if format.is_machine_readable() {
        let report = serde_json::json!({
            "error": {
                "kind": exit.label(),
                "code": exit as u8,
                "message": format!("{error}"),
            }
        });
        let mut stderr = std::io::stderr();
        let _ = serde_json::to_writer(&mut stderr, &report);
        let _ = writeln!(stderr);
    } else {
        eprintln!("carp: {error}");
    }
}

/// Start the interactive browser.
///
/// `opening` is `Some` when it should start in the protocol editor: `None`
/// inside it opens a new protocol, a path opens that one.
#[cfg(feature = "tui")]
async fn interactive(cli: &Cli, opening: Option<Option<std::path::PathBuf>>) -> Result<()> {
    use std::sync::Arc;

    use carp_client::api::CarpClient;
    use carp_client::config::Config;
    use carp_client::{Authenticator, Error};

    let config = Config::load(&cli.global.settings())?;
    let authenticator = Arc::new(Authenticator::new(&config)?);

    // The editor works on local files and needs the server only to upload, so
    // it does not demand a login on the way in. The study browser does, and
    // asking now rather than after `ratatui::init` is what keeps the URL
    // visible while the flow waits.
    if opening.is_none() {
        authenticator
            .ensure_session(|url| {
                println!("Opening {url}");
                println!("If no browser opens, visit that address to sign in.");
            })
            .await?;
    }
    let account = authenticator.account_label().await;

    let client = CarpClient::new(&config, authenticator)?;
    let cache = match db::Cache::open(&config.db_path()).await {
        Ok(cache) => cache,
        Err(error) => {
            // The cache is an optimisation; losing it must not stop the app.
            eprintln!("warning: local cache disabled ({error})");
            db::Cache::disabled()
        }
    };

    ui::icons::use_set(icon_set(cli).map_err(Error::config)?);

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

/// Which icons to draw. A rendering concern, so it is resolved here rather
/// than by the client library.
#[cfg(feature = "tui")]
fn icon_set(cli: &Cli) -> std::result::Result<ui::icons::IconSet, String> {
    let Some(value) = cli
        .global
        .icons
        .clone()
        .or_else(|| std::env::var("CARP_ICONS").ok())
    else {
        return Ok(ui::icons::IconSet::default());
    };
    ui::icons::IconSet::parse(&value)
        .ok_or_else(|| format!("unknown icon set: {value} (symbols, emoji or none)"))
}

/// Without the `tui` feature there is no browser to start.
#[cfg(not(feature = "tui"))]
async fn interactive(_cli: &Cli, _opening: Option<Option<std::path::PathBuf>>) -> Result<()> {
    color_eyre::eyre::bail!(
        "this build has no interactive browser - it was compiled without the `tui` feature.\n\
         Every study, participant, export and file is reachable from the command line; \
         try `carp --help`."
    )
}
