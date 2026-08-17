// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Session handling: keeps a valid bearer token available to the API client
//! and refreshes it transparently.

pub mod oauth;
pub mod token;

use std::path::PathBuf;

use tokio::sync::{Mutex, RwLock};

use crate::auth::token::TokenSet;
use crate::config::Config;
use crate::error::{Error, Result};

pub struct Authenticator {
    config: Config,
    http: reqwest::Client,
    token_path: PathBuf,
    tokens: RwLock<Option<TokenSet>>,
    /// Serialises refreshes so concurrent requests issue only one.
    refresh_guard: Mutex<()>,
}

impl Authenticator {
    /// Load any persisted session for this server.
    pub fn new(config: &Config) -> Result<Self> {
        let token_path = config.token_file();
        let tokens = token::load(&token_path);
        Ok(Self {
            config: config.clone(),
            http: oauth::http_client()?,
            token_path,
            tokens: RwLock::new(tokens),
            refresh_guard: Mutex::new(()),
        })
    }

    /// A usable session exists (possibly needing a refresh we can perform).
    pub async fn has_session(&self) -> bool {
        match self.tokens.read().await.as_ref() {
            Some(tokens) => !tokens.needs_refresh() || tokens.refresh_token.is_some(),
            None => false,
        }
    }

    /// Display name of the signed-in account, for the status bar.
    pub async fn account_label(&self) -> Option<String> {
        self.tokens.read().await.as_ref()?.account_label()
    }

    /// Log in if there is no usable session; a no-op otherwise.
    ///
    /// `on_url` is called with the authorization URL before the browser is
    /// opened, so a caller can show it — the browser may fail to open, or open
    /// somewhere the user cannot see.
    pub async fn ensure_session(&self, on_url: impl Fn(&str)) -> Result<()> {
        if self.has_session().await {
            return Ok(());
        }
        self.login(on_url).await
    }

    /// Run the interactive browser login and persist the result.
    pub async fn login(&self, on_url: impl Fn(&str)) -> Result<()> {
        let tokens = oauth::login(&self.config, &self.http, on_url).await?;
        token::save(&self.token_path, &tokens)?;
        *self.tokens.write().await = Some(tokens);
        Ok(())
    }

    /// Forget the stored session.
    pub async fn logout(&self) -> Result<()> {
        token::clear(&self.token_path)?;
        *self.tokens.write().await = None;
        Ok(())
    }

    /// A bearer token that is valid right now, refreshing if needed.
    pub async fn access_token(&self) -> Result<String> {
        let current = self.tokens.read().await.clone();
        let Some(current) = current else {
            return Err(Error::no_session("not signed in - run `carp auth login`"));
        };
        if !current.needs_refresh() {
            return Ok(current.access_token);
        }
        let Some(refresh_token) = current.refresh_token.clone() else {
            return Err(Error::no_session("session expired - run `carp auth login`"));
        };

        let _guard = self.refresh_guard.lock().await;
        // Another task may have refreshed while we waited for the guard.
        if let Some(tokens) = self.tokens.read().await.clone()
            && !tokens.needs_refresh()
        {
            return Ok(tokens.access_token);
        }

        let refreshed = oauth::refresh(&self.config, &self.http, &refresh_token)
            .await
            .map_err(|error| {
                Error::no_session(format!(
                    "session expired and could not be renewed ({error}) - run `carp auth login`"
                ))
            })?;
        token::save(&self.token_path, &refreshed)?;
        let access_token = refreshed.access_token.clone();
        *self.tokens.write().await = Some(refreshed);
        Ok(access_token)
    }
}
