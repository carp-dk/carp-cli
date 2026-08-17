// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! The command line.
//!
//! Nouns take verbs — `carp studies list`, `carp export download` — because the
//! set of things CARP holds is stable while what you do to them is not, and a
//! new verb should not need a new top-level word.
//!
//! Deployment selection is deliberately *not* read from the environment by
//! clap. `carp-client` resolves `--server` over `--env` over `CARP_SERVER` over
//! `CARP_ENV` over the default, and also reads `.env`; letting clap fill the
//! flags from the environment first would quietly reverse that order.

use std::path::PathBuf;

use carp_client::config::Settings;
use carp_client::time::Moment;
use clap::{Args as ClapArgs, Parser, Subcommand};

use crate::output::Format;

const ABOUT: &str = "Client for the Copenhagen Research Platform";

const AFTER_HELP: &str = "\
ENVIRONMENT:
  CARP_ENV              production (default), test or dev
  CARP_SERVER           Base URL of the CARP web service; overrides CARP_ENV
  CARP_REALM            Keycloak realm (default: Carp)
  CARP_CLIENT_ID        Public OAuth2 client id (default: carp-cli)
  CARP_DATA_DIR         Where the session and the local cache are kept
  CARP_DOWNLOAD_DIR     Where exports and study files are written
  CARP_PORTAL_URL       Base address of the CARP web portal
  CARP_ICONS            symbols (default), emoji or none
  GITHUB_TOKEN          Access to the upstream configurations repository

Each deployment keeps its own session and cache, keyed by host, so moving
between them neither signs you out of the other nor mixes their studies.
Values may also be put in a .env beside the binary.

EXAMPLES:
  carp auth login --env test
  carp studies list
  carp participants list <study> --format csv > participants.csv
  carp data query <deployment> --device Primary --type dk.cachet.carp.heartrate --from 7d
  carp export create <study> && carp export list <study>
";

#[derive(Debug, Parser)]
#[command(
    name = "carp",
    version,
    about = ABOUT,
    after_help = AFTER_HELP,
    subcommand_required = true,
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    #[command(flatten)]
    pub global: Global,
}

/// Flags that apply wherever they are written.
#[derive(Debug, Clone, Default, ClapArgs)]
pub struct Global {
    /// Deployment to talk to: production (default), test or dev
    #[arg(short = 'e', long = "env", global = true, value_name = "NAME")]
    pub environment: Option<String>,

    /// A CARP server by address, for a deployment --env does not name
    #[arg(short = 's', long, global = true, value_name = "URL")]
    pub server: Option<String>,

    /// How to print results [default: table on a terminal, json otherwise]
    #[arg(
        long,
        global = true,
        value_name = "FORMAT",
        default_value_t = Format::Auto,
        // `auto` is the literal default, but saying so twice is noise: the
        // doc comment above already says what `auto` will actually do.
        hide_default_value = true
    )]
    pub format: Format,

    /// Shorthand for --format json
    #[arg(long, global = true, conflicts_with = "format")]
    pub json: bool,

    /// Where downloads are written
    #[arg(short = 'o', long, global = true, value_name = "DIR")]
    pub output: Option<PathBuf>,

    /// Base address of the CARP web portal
    #[arg(short = 'p', long, global = true, value_name = "URL")]
    pub portal: Option<String>,

    /// Icon style for the interactive browser: symbols, emoji or none
    #[cfg(feature = "tui")]
    #[arg(short = 'i', long, global = true, value_name = "SET")]
    pub icons: Option<String>,
}

impl Global {
    /// The flags that pick a deployment and its local paths, in the shape
    /// `carp-client` resolves. Anything left `None` falls through to the
    /// environment there.
    pub fn settings(&self) -> Settings {
        Settings {
            server: self.server.clone(),
            environment: self.environment.clone(),
            data_dir: None,
            download_dir: self.output.clone(),
            portal: self.portal.clone(),
        }
    }

    /// How results should be printed, with `--json` folded in.
    pub fn format(&self) -> Format {
        if self.json { Format::Json } else { self.format }
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Sign in, sign out, and see who you are signed in as
    Auth {
        #[command(subcommand)]
        command: AuthCommand,
    },

    /// Studies you have access to
    Studies {
        #[command(subcommand)]
        command: Option<StudiesCommand>,
    },

    /// Participants enrolled in a study
    Participants {
        #[command(subcommand)]
        command: Option<ParticipantsCommand>,
    },

    /// Study deployments and the state of their devices
    Deployments {
        #[command(subcommand)]
        command: DeploymentsCommand,
    },

    /// Measurements a study has collected
    Data {
        #[command(subcommand)]
        command: DataCommand,
    },

    /// Study data exports: request one, then download it
    Export {
        #[command(subcommand)]
        command: ExportCommand,
    },

    /// Files uploaded for a study
    Files {
        #[command(subcommand)]
        command: FilesCommand,
    },

    /// Author, validate and publish study protocols
    Protocol {
        #[command(subcommand)]
        command: Option<ProtocolCommand>,
    },

    /// The interactive browser
    #[command(visible_alias = "browse")]
    Tui,

    /// Print a shell completion script
    Completions {
        /// Shell to generate for
        shell: clap_complete::Shell,
    },
}

#[derive(Debug, Subcommand)]
pub enum AuthCommand {
    /// Sign in through the browser and store the session
    Login,
    /// Forget the stored session
    Logout,
    /// Whether there is a session, and whose
    Status,
    /// Print the bearer token, for a request made by hand
    ///
    /// This is a credential: it grants everything your account can do until it
    /// expires. Prefer piping it straight into the tool that needs it.
    Token,
}

#[derive(Debug, Subcommand)]
pub enum StudiesCommand {
    /// List the studies you can see
    List {
        /// Only studies whose name or description contains this
        #[arg(long, value_name = "TEXT")]
        search: Option<String>,
    },
    /// One study in full, with its staff and participant groups
    Show {
        /// Study id
        study: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum ParticipantsCommand {
    /// List a study's participants
    List(ParticipantsList),
}

#[derive(Debug, ClapArgs)]
pub struct ParticipantsList {
    /// Study id
    pub study: String,

    /// Only participants matching this name or identity
    #[arg(long, value_name = "TEXT")]
    pub search: Option<String>,

    /// Fetch every page rather than just the first
    #[arg(long, conflicts_with = "page")]
    pub all: bool,

    /// Which page to fetch, counting from 0
    #[arg(long, value_name = "N", default_value_t = 0)]
    pub page: u32,

    /// How many participants per page
    #[arg(long, value_name = "N", default_value_t = 50)]
    pub size: u32,
}

#[derive(Debug, Subcommand)]
pub enum DeploymentsCommand {
    /// List a study's deployments and how far each has got
    List {
        /// Study id
        study: String,
    },
    /// One deployment, with every device and participant on it
    Show {
        /// Study id
        study: String,
        /// Deployment id
        deployment: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum DataCommand {
    /// How much data a study has collected, by task and day
    Summary(DataSummary),
    /// The measurements one data stream holds
    Query(DataQuery),
    /// Upload counts for one or more deployments
    Statistics {
        /// Deployment ids
        #[arg(required = true, value_name = "DEPLOYMENT")]
        deployments: Vec<String>,
    },
}

#[derive(Debug, ClapArgs)]
pub struct DataSummary {
    /// Study id
    pub study: String,

    /// Narrow to one deployment
    #[arg(long, value_name = "ID")]
    pub deployment: Option<String>,

    /// Narrow to one participant
    #[arg(long, value_name = "ID")]
    pub participant: Option<String>,

    /// Start of the window: a date, a timestamp, or an age such as 30d
    #[arg(long, value_name = "WHEN", default_value = "30d")]
    pub from: Moment,

    /// End of the window [default: now]
    #[arg(long, value_name = "WHEN")]
    pub to: Option<Moment>,

    /// Server-side grouping of the counts
    #[arg(long, default_value = "study")]
    pub scope: String,

    /// Server-side kind of summary
    #[arg(long = "kind", default_value = "daily")]
    pub kind: String,
}

#[derive(Debug, ClapArgs)]
pub struct DataQuery {
    /// Study deployment id
    pub deployment: String,

    /// Role name of the device that recorded it, as the protocol names it
    #[arg(long, value_name = "ROLE")]
    pub device: String,

    /// Data type, as `dk.cachet.carp.heartrate`
    #[arg(long = "type", value_name = "TYPE")]
    pub data_type: String,

    /// Start of the window: a date, a timestamp, or an age such as 7d
    #[arg(long, value_name = "WHEN", default_value = "7d")]
    pub from: Moment,

    /// End of the window [default: now]
    #[arg(long, value_name = "WHEN")]
    pub to: Option<Moment>,

    /// Print the server's response untouched, without interpreting it
    ///
    /// The measurement payload is not described by the OpenAPI document, so
    /// this is the way to see exactly what a deployment sent.
    #[arg(long)]
    pub raw: bool,
}

#[derive(Debug, Subcommand)]
pub enum ExportCommand {
    /// List a study's exports, newest first
    List {
        /// Study id
        study: String,
    },
    /// Ask the server to build one. It is packaged in the background.
    Create {
        /// Study id
        study: String,
        /// Restrict to these deployments [default: the whole study]
        #[arg(long, value_name = "ID")]
        deployment: Vec<String>,
        /// Only deployments that are still active
        #[arg(long)]
        active_only: bool,
        /// Wait until the archive is ready, then report it
        #[arg(long)]
        wait: bool,
    },
    /// Download a finished export
    Download {
        /// Study id
        study: String,
        /// Export id
        export: String,
    },
    /// Delete an export from the server
    Delete {
        /// Study id
        study: String,
        /// Export id
        export: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum FilesCommand {
    /// List a study's uploaded files
    List {
        /// Study id
        study: String,
        /// Server-side query expression to filter by
        #[arg(long, value_name = "QUERY")]
        query: Option<String>,
    },
    /// Download one
    Download {
        /// Study id
        study: String,
        /// File id
        file: i32,
    },
}

#[derive(Debug, Subcommand)]
pub enum ProtocolCommand {
    /// Validate a protocol; exits non-zero on any error
    #[command(visible_alias = "validate")]
    Check {
        /// A protocol.json, or a study directory holding
        /// carp/resources/protocol.json
        path: PathBuf,
    },
    /// Print a protocol's devices, tasks and schedules
    Show {
        /// A protocol.json, or a study directory
        path: PathBuf,
    },
    /// Download the upstream study configurations and record the commit
    Sync,
    /// Report what the stored catalogue holds, offline
    #[command(visible_alias = "catalogue")]
    Catalog,
    /// Open a protocol in the editor
    #[command(visible_alias = "open")]
    Edit {
        /// A protocol.json, or a study directory [default: a new protocol]
        path: Option<PathBuf>,
    },
}

#[cfg(test)]
mod tests;
