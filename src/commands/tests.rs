// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`].

use carp_client::{ApiError, Error};
use color_eyre::eyre::eyre;

use super::*;

/// The codes are a promise to whatever is calling this: a script branches on
/// them, so changing one silently breaks something downstream.
#[test]
fn the_codes_are_what_they_are_documented_as() {
    assert_eq!(Exit::Ok as u8, 0);
    assert_eq!(Exit::Failure as u8, 1);
    assert_eq!(Exit::Usage as u8, 2);
    assert_eq!(Exit::NoSession as u8, 3);
    assert_eq!(Exit::NotFound as u8, 4);
    assert_eq!(Exit::Forbidden as u8, 5);
}

fn exit_for(error: Error) -> Exit {
    Exit::of(&color_eyre::Report::new(error))
}

/// The whole point of separate codes: a caller can tell "sign in" from
/// "that does not exist" from "you may not" without reading the message.
#[test]
fn a_failure_is_classified_by_what_would_fix_it() {
    assert_eq!(
        exit_for(Error::no_session("not signed in")),
        Exit::NoSession
    );
    assert_eq!(
        exit_for(Error::login("browser never came back")),
        Exit::NoSession
    );
    assert_eq!(
        exit_for(Error::Api(ApiError::Unauthorized("expired".to_owned()))),
        Exit::NoSession
    );
    assert_eq!(
        exit_for(Error::Api(ApiError::NotFound("no such study".to_owned()))),
        Exit::NotFound
    );
    assert_eq!(
        exit_for(Error::Api(ApiError::Forbidden("not your study".to_owned()))),
        Exit::Forbidden
    );
}

/// Everything else is a plain failure. A network outage is not a missing
/// study, and must not be reported as one.
#[test]
fn anything_unclassified_is_a_plain_failure() {
    assert_eq!(
        exit_for(Error::Api(ApiError::Transport(
            "connection reset".to_owned()
        ))),
        Exit::Failure
    );
    assert_eq!(
        exit_for(Error::Api(ApiError::Decode("unexpected body".to_owned()))),
        Exit::Failure
    );
    assert_eq!(
        exit_for(Error::Api(ApiError::Status {
            status: 500,
            message: "server error".to_owned(),
        })),
        Exit::Failure
    );
    assert_eq!(
        exit_for(Error::config("unknown environment")),
        Exit::Failure
    );
    assert_eq!(Exit::of(&eyre!("something else entirely")), Exit::Failure);
}

/// An `ApiError` can reach `main` without a `carp_client::Error` around it,
/// because the endpoints return it directly. It has to classify the same
/// either way, or the code would depend on which call site failed.
#[test]
fn an_unwrapped_api_error_classifies_the_same() {
    let wrapped = exit_for(Error::Api(ApiError::NotFound("gone".to_owned())));
    let bare = Exit::of(&color_eyre::Report::new(ApiError::NotFound(
        "gone".to_owned(),
    )));
    assert_eq!(wrapped, bare);
    assert_eq!(bare, Exit::NotFound);

    for (error, expected) in [
        (ApiError::Unauthorized(String::new()), Exit::NoSession),
        (ApiError::Forbidden(String::new()), Exit::Forbidden),
        (ApiError::Io(String::new()), Exit::Failure),
    ] {
        assert_eq!(Exit::of(&color_eyre::Report::new(error)), expected);
    }
}

/// The JSON error report names the kind, so a caller can branch on the word
/// instead of the number. Every code needs one, and no two may share it.
#[test]
fn every_code_has_its_own_name() {
    let all = [
        Exit::Ok,
        Exit::Failure,
        Exit::Usage,
        Exit::NoSession,
        Exit::NotFound,
        Exit::Forbidden,
    ];
    let labels: std::collections::HashSet<_> = all.iter().map(|exit| exit.label()).collect();
    assert_eq!(labels.len(), all.len(), "two codes share a name");
    assert!(all.iter().all(|exit| !exit.label().is_empty()));
}
