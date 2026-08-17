// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Command line parsing. The CLI is intentionally tiny: the TUI is the
//! interface, flags only pick the deployment and manage the session.

use std::path::PathBuf;

use color_eyre::Result;
use color_eyre::eyre::bail;

pub const HELP: &str = "\
carp - terminal client for the Copenhagen Research Platform

USAGE:
    carp [OPTIONS] [COMMAND]

COMMANDS:
    tui                 Start the terminal UI (default)
    login               Authenticate and store the session, then exit
    logout              Forget the stored session, then exit

    protocol            Open the protocol editor
    protocol sync       Download the upstream study configurations, which the
                        editor learns its vocabulary from, and record the
                        commit they came from
    protocol catalog    Show what the stored catalogue holds, offline
    protocol check <P>  Validate a protocol; exits non-zero on any error, so it
                        can be used as a pre-commit or CI check
    protocol show <P>   Print a protocol's devices, tasks and schedules
    protocol edit <P>   Open a protocol in the editor

    <P> is a protocol.json, or a study directory containing
    carp/resources/protocol.json.

OPTIONS:
    -e, --env <NAME>    Deployment: production (default), test or dev
                        (env CARP_ENV)
    -s, --server <URL>  A CARP server by address, for a deployment --env does
                        not name; overrides it (env CARP_SERVER)
    -o, --output <DIR>  Download directory (env CARP_DOWNLOAD_DIR)
    -p, --portal <URL>  CARP web portal opened by `o` (env CARP_PORTAL_URL)
    -i, --icons <SET>   symbols (default), emoji or none (env CARP_ICONS)
    -h, --help          Print this help
    -V, --version       Print version

ENVIRONMENT:
    GITHUB_TOKEN        Raises the GitHub API rate limit used by
                        `protocol sync`; only needed on a shared address
    CARP_ENV            Which deployment to talk to:
                          production  https://carp.computerome.dk  (default)
                          test        https://test.carp.dk
                          dev         https://dev.carp.dk
                        Each keeps its own session and cache, so switching
                        neither logs you out of the other nor mixes them up.
    CARP_SERVER         Base URL of the CARP web service; overrides CARP_ENV
    CARP_REALM          Keycloak realm (default: Carp)
    CARP_CLIENT_ID      Public OAuth2 client id (default: carp-cli)
    CARP_DATA_DIR       Where tokens and the local cache are stored
    CARP_DOWNLOAD_DIR   Where exports and study files are written
    CARP_PORTAL_URL     Base address of the CARP web portal; discovered from
                        the server when unset
    CARP_ICONS          Icon style: symbols (single width, works everywhere),
                        emoji (double width, needs a capable terminal), none
    CARP_PORTAL_STUDY_PATH
                        Path of a study in the portal, {study} standing for
                        the study id (default: /studies/{study})
";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Tui,
    Login,
    Logout,
    Help,
    Version,
    /// Open the editor, on `path` when one was given.
    Protocol {
        path: Option<PathBuf>,
    },
    /// Download the upstream configurations.
    ProtocolSync,
    /// Report what the stored catalogue holds.
    ProtocolCatalog,
    /// Validate a protocol and exit non-zero when it has errors.
    ProtocolCheck {
        path: PathBuf,
    },
    /// Print a protocol's shape.
    ProtocolShow {
        path: PathBuf,
    },
}

#[derive(Debug, Clone)]
pub struct Args {
    pub command: Command,
    pub server: Option<String>,
    /// A deployment by name; see `carp::config::Environment`.
    pub environment: Option<String>,
    pub download_dir: Option<PathBuf>,
    pub portal: Option<String>,
    pub icons: Option<String>,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            command: Command::Tui,
            server: None,
            environment: None,
            download_dir: None,
            portal: None,
            icons: None,
        }
    }
}

impl Args {
    pub fn parse() -> Result<Self> {
        Self::from_iter(std::env::args().skip(1))
    }

    pub fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Result<Self> {
        let mut args = Self::default();
        let mut iter = iter.into_iter();

        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "-h" | "--help" => args.command = Command::Help,
                "-V" | "--version" => args.command = Command::Version,
                "-s" | "--server" => {
                    let Some(value) = iter.next() else {
                        bail!("--server requires a URL");
                    };
                    args.server = Some(value);
                }
                "-e" | "--env" | "--environment" => {
                    let Some(value) = iter.next() else {
                        bail!("--env requires production, test or dev");
                    };
                    args.environment = Some(value);
                }
                "-o" | "--output" => {
                    let Some(value) = iter.next() else {
                        bail!("--output requires a directory");
                    };
                    args.download_dir = Some(PathBuf::from(value));
                }
                "-p" | "--portal" => {
                    let Some(value) = iter.next() else {
                        bail!("--portal requires a URL");
                    };
                    args.portal = Some(value);
                }
                "-i" | "--icons" => {
                    let Some(value) = iter.next() else {
                        bail!("--icons requires symbols, emoji or none");
                    };
                    args.icons = Some(value);
                }
                "tui" => args.command = Command::Tui,
                "login" => args.command = Command::Login,
                "logout" => args.command = Command::Logout,
                // `protocol` takes a subcommand, or a path, or neither. The
                // remaining arguments are consumed here rather than by the
                // outer loop, so a path is not mistaken for a flag.
                "protocol" => args.command = protocol_command(&mut iter)?,
                other => bail!("unknown argument: {other}\n\n{HELP}"),
            }
        }

        Ok(args)
    }
}

/// Parse whatever follows `protocol`.
fn protocol_command<I: Iterator<Item = String>>(iter: &mut I) -> Result<Command> {
    let Some(argument) = iter.next() else {
        return Ok(Command::Protocol { path: None });
    };

    // A subcommand needing a path says so by name, so a missing one is an
    // error rather than a silent no-op.
    let path_for = |iter: &mut I, subcommand: &str| -> Result<PathBuf> {
        match iter.next() {
            Some(path) => Ok(PathBuf::from(path)),
            None => bail!(
                "`protocol {subcommand}` needs a path to a protocol.json or a study directory"
            ),
        }
    };

    Ok(match argument.as_str() {
        "sync" => Command::ProtocolSync,
        "catalog" | "catalogue" => Command::ProtocolCatalog,
        "check" | "validate" => Command::ProtocolCheck {
            path: path_for(iter, "check")?,
        },
        "show" => Command::ProtocolShow {
            path: path_for(iter, "show")?,
        },
        "edit" | "open" => Command::Protocol {
            path: Some(path_for(iter, "edit")?),
        },
        // Anything else is taken as a path, so `carp protocol my-study` works.
        other if !other.starts_with('-') => Command::Protocol {
            path: Some(PathBuf::from(other)),
        },
        other => bail!("unknown protocol subcommand: {other}\n\n{HELP}"),
    })
}

#[cfg(test)]
mod tests;
