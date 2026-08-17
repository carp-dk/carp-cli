// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! What can go wrong before, or instead of, a request.
//!
//! A library reports; it does not decide what to do about it. The variants are
//! therefore separated by what a caller would do differently rather than by
//! where they were raised: a missing session is worth offering a login, a bad
//! setting is worth correcting, and a failed write is neither. The command line
//! turns them into exit codes and the Python module into exception types, so
//! collapsing two of these into one string would cost both.

use std::path::Path;

use crate::api::error::ApiError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The server was reached and answered with a failure, or could not be
    /// reached at all. See [`ApiError`] for which.
    #[error(transparent)]
    Api(#[from] ApiError),

    /// A setting names something that does not exist - an unknown deployment,
    /// an address that is not a URL. Nothing was attempted.
    #[error("{0}")]
    Config(String),

    /// There is no usable session. Distinct from [`ApiError::Unauthorized`],
    /// which is the server rejecting one we believed in: this is knowing
    /// before asking, and it is the case where offering a login is right.
    #[error("{0}")]
    NoSession(String),

    /// The browser login was started and did not complete.
    #[error("{0}")]
    Login(String),

    /// Reading or writing local state failed - the token file, a download.
    #[error("{context}: {source}")]
    Io {
        context: String,
        #[source]
        source: std::io::Error,
    },
}

impl Error {
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }

    pub fn no_session(message: impl Into<String>) -> Self {
        Self::NoSession(message.into())
    }

    pub fn login(message: impl Into<String>) -> Self {
        Self::Login(message.into())
    }

    /// An I/O failure, said in terms of what was being attempted. `io::Error`
    /// alone gives "permission denied" without naming the file.
    pub fn io(context: impl Into<String>, source: std::io::Error) -> Self {
        Self::Io {
            context: context.into(),
            source,
        }
    }

    /// The same, for the common case of a failure against a known path.
    pub fn at_path(action: &str, path: &Path, source: std::io::Error) -> Self {
        Self::io(format!("{action} {}", path.display()), source)
    }

    /// True when signing in is what would fix this - whether we knew there was
    /// no session, or the server told us.
    pub fn needs_login(&self) -> bool {
        matches!(
            self,
            Self::NoSession(_) | Self::Login(_) | Self::Api(ApiError::Unauthorized(_))
        )
    }
}

/// Adds the "what was being attempted" half of an [`Error::Io`] to a plain
/// `io::Result`, so call sites read as one expression.
pub(crate) trait IoContext<T> {
    fn at(self, action: &str, path: &Path) -> Result<T>;
}

impl<T> IoContext<T> for std::io::Result<T> {
    fn at(self, action: &str, path: &Path) -> Result<T> {
        self.map_err(|source| Error::at_path(action, path, source))
    }
}
