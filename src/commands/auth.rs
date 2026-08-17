// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! `carp auth` - the session against one deployment.

use carp_client::config::Config;
use carp_client::{Authenticator, Error};
use color_eyre::Result;
use serde::Serialize;

use crate::cli::{AuthCommand, Global};
use crate::commands::server_label;
use crate::output::{self, Format};

pub async fn run(command: &AuthCommand, global: &Global) -> Result<()> {
    let config = Config::load(&global.settings())?;
    let authenticator = Authenticator::new(&config)?;
    let format = global.format();

    match command {
        AuthCommand::Login => login(&authenticator, &config, format).await,
        AuthCommand::Logout => logout(&authenticator, &config, format).await,
        AuthCommand::Status => status(&authenticator, &config, format).await,
        AuthCommand::Token => token(&authenticator).await,
    }
}

/// What `status` reports, and what `login` confirms.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionInfo {
    server: String,
    signed_in: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    account: Option<String>,
}

impl SessionInfo {
    async fn read(authenticator: &Authenticator, config: &Config) -> Self {
        Self {
            server: server_label(config).to_owned(),
            signed_in: authenticator.has_session().await,
            account: authenticator.account_label().await,
        }
    }

    fn lines(&self) -> Vec<(&'static str, String)> {
        let mut lines = vec![("server", self.server.clone())];
        match (&self.signed_in, &self.account) {
            (true, Some(account)) => lines.push(("account", account.clone())),
            (true, None) => lines.push(("account", "signed in".to_owned())),
            (false, _) => lines.push(("account", "not signed in".to_owned())),
        }
        lines
    }
}

async fn login(authenticator: &Authenticator, config: &Config, format: Format) -> Result<()> {
    // The URL and the wait go to stderr: `carp auth login --json` should still
    // produce nothing but the result on stdout.
    authenticator
        .login(|url| {
            output::note(format!("Opening {url}"));
            output::note("If no browser opens, visit that address to sign in.");
        })
        .await?;

    let info = SessionInfo::read(authenticator, config).await;
    output::detail(&info, &info.lines(), format)
}

async fn logout(authenticator: &Authenticator, config: &Config, format: Format) -> Result<()> {
    authenticator.logout().await?;
    let info = SessionInfo::read(authenticator, config).await;
    output::detail(&info, &info.lines(), format)
}

async fn status(authenticator: &Authenticator, config: &Config, format: Format) -> Result<()> {
    let info = SessionInfo::read(authenticator, config).await;
    let signed_in = info.signed_in;
    output::detail(&info, &info.lines(), format)?;

    // Exits non-zero when there is no session, so `carp auth status` works as
    // a test in a script rather than only as something to read.
    if signed_in {
        Ok(())
    } else {
        Err(Error::no_session(format!(
            "not signed in to {} - run `carp auth login`",
            server_label(config)
        ))
        .into())
    }
}

/// Print the bearer token, refreshing it first if it is near expiry.
///
/// Never formatted as JSON or decorated: the only use for this is to be piped
/// into something else, and anything printed alongside it would end up in the
/// header.
async fn token(authenticator: &Authenticator) -> Result<()> {
    println!("{}", authenticator.access_token().await?);
    Ok(())
}
