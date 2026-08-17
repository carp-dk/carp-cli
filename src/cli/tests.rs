// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`].

use clap::CommandFactory;

use super::*;

fn parse(arguments: &[&str]) -> Result<Cli, clap::Error> {
    Cli::try_parse_from(std::iter::once("carp").chain(arguments.iter().copied()))
}

/// clap's own consistency checks: conflicting flags, duplicate names, a
/// `default_value` that its own parser would reject.
#[test]
fn the_command_surface_is_well_formed() {
    Cli::command().debug_assert();
}

/// Bare `carp` used to open the browser. It prints help now, because the
/// command line is the interface and the browser is one of its commands.
#[test]
fn bare_carp_asks_for_a_command() {
    let error = parse(&[]).unwrap_err();
    assert_eq!(
        error.kind(),
        clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
    );
    // And the browser is still reachable, by name.
    assert!(matches!(parse(&["tui"]).unwrap().command, Command::Tui));
}

#[test]
fn protocol_subcommands_parse() {
    assert!(matches!(
        parse(&["protocol", "sync"]).unwrap().command,
        Command::Protocol {
            command: Some(ProtocolCommand::Sync)
        }
    ));
    assert!(matches!(
        parse(&["protocol", "catalog"]).unwrap().command,
        Command::Protocol {
            command: Some(ProtocolCommand::Catalog)
        }
    ));
    // The aliases the previous parser accepted still work.
    assert!(matches!(
        parse(&["protocol", "catalogue"]).unwrap().command,
        Command::Protocol {
            command: Some(ProtocolCommand::Catalog)
        }
    ));
    assert!(matches!(
        parse(&["protocol", "validate", "study"]).unwrap().command,
        Command::Protocol {
            command: Some(ProtocolCommand::Check { .. })
        }
    ));

    let Command::Protocol {
        command: Some(ProtocolCommand::Check { path }),
    } = parse(&["protocol", "check", "study"]).unwrap().command
    else {
        panic!("check did not parse")
    };
    assert_eq!(path, PathBuf::from("study"));
}

/// `carp protocol` alone opens the editor on a new protocol, and `edit <path>`
/// opens an existing one.
#[test]
fn protocol_opens_the_editor() {
    assert!(matches!(
        parse(&["protocol"]).unwrap().command,
        Command::Protocol { command: None }
    ));

    let Command::Protocol {
        command: Some(ProtocolCommand::Edit { path }),
    } = parse(&["protocol", "edit", "neuropathy"]).unwrap().command
    else {
        panic!("edit did not parse")
    };
    assert_eq!(path, Some(PathBuf::from("neuropathy")));
}

/// A subcommand that needs a path has to say so rather than doing nothing.
#[test]
fn a_missing_path_is_an_error() {
    let error = parse(&["protocol", "check"]).unwrap_err();
    assert_eq!(
        error.kind(),
        clap::error::ErrorKind::MissingRequiredArgument
    );
}

/// Global flags are global: they parse before a command, after it, and after
/// its arguments, so nobody has to remember where they go.
#[test]
fn global_flags_parse_wherever_they_are_written() {
    for arguments in [
        &["--server", "https://dev.carp.dk", "protocol", "sync"][..],
        &["protocol", "sync", "--server", "https://dev.carp.dk"][..],
    ] {
        let cli = parse(arguments).unwrap();
        assert!(matches!(
            cli.command,
            Command::Protocol {
                command: Some(ProtocolCommand::Sync)
            }
        ));
        assert_eq!(cli.global.server.as_deref(), Some("https://dev.carp.dk"));
    }

    let cli = parse(&["studies", "show", "abc", "--format", "csv"]).unwrap();
    assert_eq!(cli.global.format, Format::Csv);
}

#[test]
fn a_deployment_can_be_named_instead_of_addressed() {
    assert_eq!(
        parse(&["--env", "test", "tui"])
            .unwrap()
            .global
            .environment
            .as_deref(),
        Some("test")
    );
    assert_eq!(
        parse(&["-e", "dev", "tui"])
            .unwrap()
            .global
            .environment
            .as_deref(),
        Some("dev")
    );
    assert_eq!(parse(&["tui"]).unwrap().global.environment, None);

    // The name is not validated here; carp-client owns which names exist.
    let cli = parse(&["-e", "prod", "protocol", "sync"]).unwrap();
    assert_eq!(cli.global.environment.as_deref(), Some("prod"));
}

#[test]
fn a_flag_without_its_value_is_an_error() {
    assert!(parse(&["--env"]).is_err());
    assert!(parse(&["--server"]).is_err());
    assert!(parse(&["--format"]).is_err());
}

#[test]
fn an_unknown_command_is_refused() {
    assert!(parse(&["protocol", "--nonsense"]).is_err());
    assert!(parse(&["nonsense"]).is_err());
    assert!(parse(&["studies", "nonsense"]).is_err());
    assert!(parse(&["--format", "yaml", "studies"]).is_err());
}

/// The listing commands take a bare noun, because that is the only thing they
/// could mean.
#[test]
fn a_bare_noun_lists() {
    assert!(matches!(
        parse(&["studies"]).unwrap().command,
        Command::Studies { command: None }
    ));
    assert!(matches!(
        parse(&["studies", "list"]).unwrap().command,
        Command::Studies {
            command: Some(StudiesCommand::List { search: None })
        }
    ));
}

/// `--json` is shorthand for `--format json`, and saying both is a mistake
/// worth catching rather than silently resolving.
#[test]
fn json_is_shorthand_for_the_format() {
    let cli = parse(&["studies", "--json"]).unwrap();
    assert_eq!(cli.global.format(), Format::Json);

    // Without either, the format is decided by where the output is going.
    assert_eq!(parse(&["studies"]).unwrap().global.format(), Format::Auto);

    assert!(parse(&["studies", "--json", "--format", "csv"]).is_err());
}

#[test]
fn a_data_query_names_the_stream_it_wants() {
    let Command::Data {
        command: DataCommand::Query(query),
    } = parse(&[
        "data",
        "query",
        "df98d925-3ab4-4b78-8139-fea86d809dc5",
        "--device",
        "Primary Phone",
        "--type",
        "dk.cachet.carp.heartrate",
        "--from",
        "2026-08-01",
    ])
    .unwrap()
    .command
    else {
        panic!("data query did not parse")
    };

    assert_eq!(query.deployment, "df98d925-3ab4-4b78-8139-fea86d809dc5");
    assert_eq!(query.device, "Primary Phone");
    assert_eq!(query.data_type, "dk.cachet.carp.heartrate");
    assert_eq!(
        query.from.resolve().to_rfc3339(),
        "2026-08-01T00:00:00+00:00"
    );
    // Left open, meaning now.
    assert!(query.to.is_none());
    assert!(!query.raw);

    // The stream cannot be guessed, so both parts of it are required.
    assert!(parse(&["data", "query", "abc"]).is_err());
    assert!(parse(&["data", "query", "abc", "--device", "Primary"]).is_err());
}

/// A window given as an age is the common case, so it has to parse as an
/// argument and not only as a string.
#[test]
fn a_window_may_be_given_as_an_age() {
    let Command::Data {
        command: DataCommand::Query(query),
    } = parse(&[
        "data", "query", "abc", "--device", "Primary", "--type", "x.y", "--from", "7d",
    ])
    .unwrap()
    .command
    else {
        panic!("data query did not parse")
    };
    assert_eq!(
        query.from,
        carp_client::time::Moment::Ago(chrono::Duration::days(7))
    );

    assert!(
        parse(&[
            "data",
            "query",
            "abc",
            "--device",
            "P",
            "--type",
            "x.y",
            "--from",
            "yesterday"
        ])
        .is_err()
    );
}

/// Nothing that reads a study should be able to change one by accident, so the
/// verbs that write are the ones spelled out.
#[test]
fn every_command_group_is_reachable() {
    for arguments in [
        &["auth", "login"][..],
        &["auth", "logout"][..],
        &["auth", "status"][..],
        &["auth", "token"][..],
        &["studies", "list"][..],
        &["studies", "show", "s"][..],
        &["participants", "list", "s"][..],
        &["deployments", "list", "s"][..],
        &["deployments", "show", "s", "d"][..],
        &["data", "summary", "s"][..],
        &["data", "statistics", "d"][..],
        &["export", "list", "s"][..],
        &["export", "create", "s"][..],
        &["export", "download", "s", "e"][..],
        &["export", "delete", "s", "e"][..],
        &["files", "list", "s"][..],
        &["files", "download", "s", "7"][..],
        &["protocol", "show", "p"][..],
        &["tui"][..],
        &["browse"][..],
        &["completions", "zsh"][..],
    ] {
        assert!(parse(arguments).is_ok(), "{arguments:?} did not parse");
    }
}

/// `--all` walks every page; `--page` asks for one. Wanting both is a
/// contradiction, not a preference.
#[test]
fn paging_flags_that_contradict_are_refused() {
    assert!(parse(&["participants", "list", "s", "--all"]).is_ok());
    assert!(parse(&["participants", "list", "s", "--page", "2"]).is_ok());
    assert!(parse(&["participants", "list", "s", "--all", "--page", "2"]).is_err());
}
