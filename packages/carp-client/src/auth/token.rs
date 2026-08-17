// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Persisted OAuth2 session.

use std::fs;
use std::path::Path;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{Error, IoContext, Result};

/// Refresh this long before the access token actually expires, so a request
/// never races the expiry.
const REFRESH_SKEW_SECONDS: i64 = 60;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSet {
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub expires_at: Option<DateTime<Utc>>,
}

impl TokenSet {
    pub fn new(
        access_token: String,
        refresh_token: Option<String>,
        expires_in: Option<std::time::Duration>,
    ) -> Self {
        let expires_at = expires_in
            .and_then(|d| Duration::from_std(d).ok())
            .map(|d| Utc::now() + d);
        Self {
            access_token,
            refresh_token,
            expires_at,
        }
    }

    /// True when the access token is expired, or close enough that it should
    /// be refreshed before the next request.
    pub fn needs_refresh(&self) -> bool {
        match self.expires_at {
            Some(expiry) => Utc::now() + Duration::seconds(REFRESH_SKEW_SECONDS) >= expiry,
            // Without an expiry we cannot know; assume it is still good and let
            // a 401 drive the refresh.
            None => false,
        }
    }

    /// Best-effort display name of the signed-in account, read from the JWT
    /// payload. Purely cosmetic: the server is the authority on identity.
    pub fn account_label(&self) -> Option<String> {
        let payload = self.access_token.split('.').nth(1)?;
        let decoded = URL_SAFE_NO_PAD.decode(payload).ok()?;
        let claims: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
        for key in ["preferred_username", "email", "name", "sub"] {
            if let Some(value) = claims.get(key).and_then(|v| v.as_str()) {
                return Some(value.to_owned());
            }
        }
        None
    }
}

/// Read the token file, if one exists. A malformed file is treated as "no
/// session" rather than a hard error, so a bad file cannot lock the user out.
pub fn load(path: &Path) -> Option<TokenSet> {
    let contents = fs::read_to_string(path).ok()?;
    serde_json::from_str(&contents).ok()
}

pub fn save(path: &Path, tokens: &TokenSet) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).at("creating", parent)?;
    }
    let json = serde_json::to_string_pretty(tokens)
        .map_err(|error| Error::login(format!("serialising the session: {error}")))?;
    fs::write(path, json).at("writing", path)?;
    restrict_permissions(path);
    Ok(())
}

pub fn clear(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(Error::at_path("removing", path, err)),
    }
}

#[cfg(unix)]
fn restrict_permissions(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
}

#[cfg(not(unix))]
fn restrict_permissions(_path: &Path) {}
