// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`].

use super::*;

fn parse(arguments: &[&str]) -> Result<Args> {
    Args::from_iter(arguments.iter().map(|argument| (*argument).to_owned()))
}

#[test]
fn the_default_command_is_the_interface() {
    assert_eq!(parse(&[]).unwrap().command, Command::Tui);
}

#[test]
fn protocol_subcommands_parse() {
    assert_eq!(
        parse(&["protocol", "sync"]).unwrap().command,
        Command::ProtocolSync
    );
    assert_eq!(
        parse(&["protocol", "catalog"]).unwrap().command,
        Command::ProtocolCatalog
    );
    assert_eq!(
        parse(&["protocol", "check", "study"]).unwrap().command,
        Command::ProtocolCheck {
            path: PathBuf::from("study")
        }
    );
    assert_eq!(
        parse(&["protocol", "show", "study"]).unwrap().command,
        Command::ProtocolShow {
            path: PathBuf::from("study")
        }
    );
}

/// `carp protocol` alone opens the editor, and a bare path opens that
/// protocol in it.
#[test]
fn protocol_opens_the_editor() {
    assert_eq!(
        parse(&["protocol"]).unwrap().command,
        Command::Protocol { path: None }
    );
    assert_eq!(
        parse(&["protocol", "neuropathy"]).unwrap().command,
        Command::Protocol {
            path: Some(PathBuf::from("neuropathy"))
        }
    );
    assert_eq!(
        parse(&["protocol", "edit", "neuropathy"]).unwrap().command,
        Command::Protocol {
            path: Some(PathBuf::from("neuropathy"))
        }
    );
}

/// A subcommand that needs a path has to say so rather than doing
/// nothing.
#[test]
fn a_missing_path_is_an_error() {
    let error = parse(&["protocol", "check"]).unwrap_err().to_string();
    assert!(error.contains("needs a path"), "{error}");
}

#[test]
fn flags_still_parse_alongside_a_command() {
    let args = parse(&["--server", "https://dev.carp.dk", "protocol", "sync"]).unwrap();
    assert_eq!(args.command, Command::ProtocolSync);
    assert_eq!(args.server.as_deref(), Some("https://dev.carp.dk"));
}

#[test]
fn an_unknown_subcommand_is_refused() {
    assert!(parse(&["protocol", "--nonsense"]).is_err());
    assert!(parse(&["nonsense"]).is_err());
}
