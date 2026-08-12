// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Runtime configuration: which CARP deployment to talk to and where to keep
//! local state (tokens, cache database, downloaded data).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use color_eyre::Result;
use color_eyre::eyre::{Context, eyre};
use url::Url;

use crate::cli::Args;
use crate::ui::icons::IconSet;

pub const DEFAULT_SERVER: &str = "https://carp.computerome.dk";
pub const DEFAULT_CLIENT_ID: &str = "carp-cli";
pub const DEFAULT_REALM: &str = "Carp";
/// Path of a study in the CARP web portal, relative to its base address.
/// `{study}` is replaced with the study id.
pub const DEFAULT_PORTAL_STUDY_PATH: &str = "/studies/{study}";

/// Everything the app needs to know before it starts talking to CARP.
#[derive(Debug, Clone)]
pub struct Config {
    /// Base URL of the CARP web service, e.g. `https://dev.carp.dk`.
    pub server: Url,
    /// Keycloak realm hosting the CARP users.
    pub realm: String,
    /// Public OAuth2 client id used for the authorization-code + PKCE flow.
    pub client_id: String,
    /// Directory holding the token file and the cache database.
    pub data_dir: PathBuf,
    /// Directory that exports and study files are written to.
    pub download_dir: PathBuf,
    /// Base address of the CARP web portal, when it is known up front.
    /// Discovered from the server otherwise.
    pub portal_url: Option<Url>,
    /// Path template for a study in the portal.
    pub portal_study_path: String,
    /// Which icon set the interface draws.
    pub icons: IconSet,
}

impl Config {
    /// Resolve configuration from (in order of precedence) CLI flags,
    /// environment variables, `.env` in the working directory, and defaults.
    pub fn load(args: &Args) -> Result<Self> {
        let dotenv = read_dotenv(Path::new(".env"));
        let setting = |key: &str| -> Option<String> {
            std::env::var(key).ok().or_else(|| dotenv.get(key).cloned())
        };

        let server = args
            .server
            .clone()
            .or_else(|| setting("CARP_SERVER"))
            .unwrap_or_else(|| DEFAULT_SERVER.to_owned());
        let server = Url::parse(server.trim_end_matches('/'))
            .with_context(|| format!("invalid CARP server URL: {server}"))?;

        let realm = setting("CARP_REALM").unwrap_or_else(|| DEFAULT_REALM.to_owned());
        let client_id = setting("CARP_CLIENT_ID").unwrap_or_else(|| DEFAULT_CLIENT_ID.to_owned());

        let data_dir = match setting("CARP_DATA_DIR") {
            Some(dir) => PathBuf::from(dir),
            None => dirs::data_dir()
                .ok_or_else(|| eyre!("cannot determine a data directory for this platform"))?
                .join("carp"),
        };

        let download_dir = args
            .download_dir
            .clone()
            .or_else(|| setting("CARP_DOWNLOAD_DIR").map(PathBuf::from))
            .unwrap_or_else(|| {
                dirs::download_dir()
                    .unwrap_or_else(|| data_dir.clone())
                    .join("carp")
            });

        let portal_url = args
            .portal
            .clone()
            .or_else(|| setting("CARP_PORTAL_URL"))
            .map(|url| Url::parse(&url).with_context(|| format!("invalid CARP portal URL: {url}")))
            .transpose()?;
        let portal_study_path = setting("CARP_PORTAL_STUDY_PATH")
            .unwrap_or_else(|| DEFAULT_PORTAL_STUDY_PATH.to_owned());

        let icons = args
            .icons
            .clone()
            .or_else(|| setting("CARP_ICONS"))
            .map(|value| {
                IconSet::parse(&value)
                    .ok_or_else(|| eyre!("unknown icon set: {value} (symbols, emoji or none)"))
            })
            .transpose()?
            .unwrap_or_default();

        fs::create_dir_all(&data_dir)
            .with_context(|| format!("creating data directory {}", data_dir.display()))?;

        Ok(Self {
            server,
            realm,
            client_id,
            data_dir,
            download_dir,
            portal_url,
            portal_study_path,
            icons,
        })
    }

    /// Keycloak authorization endpoint.
    pub fn auth_url(&self) -> String {
        format!(
            "{}/auth/realms/{}/protocol/openid-connect/auth",
            self.server.as_str().trim_end_matches('/'),
            self.realm
        )
    }

    /// Keycloak token endpoint.
    pub fn token_url(&self) -> String {
        format!(
            "{}/auth/realms/{}/protocol/openid-connect/token",
            self.server.as_str().trim_end_matches('/'),
            self.realm
        )
    }

    /// Where the OAuth2 tokens for this server are persisted.
    pub fn token_file(&self) -> PathBuf {
        self.data_dir.join(format!("tokens-{}.json", self.slug()))
    }

    /// Where the local cache database lives.
    pub fn db_path(&self) -> PathBuf {
        self.data_dir.join(format!("cache-{}.db", self.slug()))
    }

    /// Host name, safe for use in a file name, so several deployments
    /// (dev/staging/prod) can be used side by side.
    fn slug(&self) -> String {
        self.server
            .host_str()
            .unwrap_or("carp")
            .replace(['.', ':'], "-")
    }
}

/// Minimal `.env` reader: `KEY=VALUE` per line, `#` comments, no interpolation.
///
/// The values are returned rather than exported: mutating the environment of a
/// running process is unsound once other threads exist, and only this module
/// needs them.
fn read_dotenv(path: &Path) -> HashMap<String, String> {
    let Ok(contents) = fs::read_to_string(path) else {
        return HashMap::new();
    };
    contents
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| {
            (
                key.trim().trim_start_matches("export ").trim().to_owned(),
                value.trim().trim_matches(['"', '\'']).to_owned(),
            )
        })
        .filter(|(key, _)| !key.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dotenv_is_parsed() {
        let path = std::env::temp_dir().join("carp-cli-dotenv-test");
        fs::write(
            &path,
            "# comment\n\nexport CARP_CLIENT_ID = \"carp-cli\"\nCARP_REALM=Carp\nbroken\n",
        )
        .unwrap();

        let values = read_dotenv(&path);
        assert_eq!(values.get("CARP_CLIENT_ID").unwrap(), "carp-cli");
        assert_eq!(values.get("CARP_REALM").unwrap(), "Carp");
        assert_eq!(values.len(), 2);
    }

    #[test]
    fn missing_dotenv_is_not_an_error() {
        assert!(read_dotenv(Path::new("/nonexistent/.env")).is_empty());
    }
}
