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

/// The CARP deployments the CLI knows by name. Any other one is still
/// reachable by giving its address to `--server`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    /// Live data and live participants.
    Production,
    /// The staging deployment, which is what production becomes next.
    Test,
    /// Where unreleased server work lands first.
    Dev,
}

impl Environment {
    /// In the order a change travels through them.
    pub const ALL: [Self; 3] = [Self::Dev, Self::Test, Self::Production];

    /// Accepts the shorthands people actually type for these.
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "production" | "prod" => Some(Self::Production),
            "test" | "staging" | "stage" => Some(Self::Test),
            "dev" | "development" => Some(Self::Dev),
            _ => None,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Production => "production",
            Self::Test => "test",
            Self::Dev => "dev",
        }
    }

    pub const fn url(self) -> &'static str {
        match self {
            Self::Production => "https://carp.computerome.dk",
            Self::Test => "https://test.carp.dk",
            Self::Dev => "https://dev.carp.dk",
        }
    }

    /// The accepted names, for an error message that says what to type.
    fn names() -> String {
        Self::ALL
            .iter()
            .map(|environment| environment.name())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// What a build talks to when nothing selects otherwise. Released binaries
/// carry this, so it has to stay production - `the_default_is_production`
/// fails the release if it ever does not.
pub const DEFAULT_ENVIRONMENT: Environment = Environment::Production;
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

        let server_setting = setting("CARP_SERVER");
        let environment_setting = setting("CARP_ENV");
        let server = resolve_server(
            args.server.as_deref(),
            args.environment.as_deref(),
            server_setting.as_deref(),
            environment_setting.as_deref(),
        )?;
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

/// Which deployment to talk to.
///
/// An address is more specific than a name, and a flag is more deliberate than
/// something inherited from the environment, so they rank in that order. The
/// last resort is [`DEFAULT_ENVIRONMENT`], never whatever was used last: a
/// command that does not say which deployment it means has to be the safe one
/// to guess at.
fn resolve_server(
    server_flag: Option<&str>,
    environment_flag: Option<&str>,
    server_setting: Option<&str>,
    environment_setting: Option<&str>,
) -> Result<String> {
    let named = |value: &str| -> Result<String> {
        Environment::parse(value)
            .map(|environment| environment.url().to_owned())
            .ok_or_else(|| {
                eyre!(
                    "unknown CARP environment: {value} (expected {})",
                    Environment::names()
                )
            })
    };

    if let Some(url) = server_flag {
        return Ok(url.to_owned());
    }
    if let Some(name) = environment_flag {
        return named(name);
    }
    if let Some(url) = server_setting {
        return Ok(url.to_owned());
    }
    if let Some(name) = environment_setting {
        return named(name);
    }
    Ok(DEFAULT_ENVIRONMENT.url().to_owned())
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

    /// The release workflow builds whatever this constant says, so a binary
    /// handed to a researcher points at live CARP. Changing the default to a
    /// test deployment for local convenience would ship that to everyone;
    /// this test is what stops it reaching a release.
    #[test]
    fn the_default_is_production() {
        assert_eq!(DEFAULT_ENVIRONMENT, Environment::Production);
        assert_eq!(
            resolve_server(None, None, None, None).unwrap(),
            "https://carp.computerome.dk"
        );
    }

    #[test]
    fn each_environment_has_its_own_address() {
        assert_eq!(Environment::Production.url(), "https://carp.computerome.dk");
        assert_eq!(Environment::Test.url(), "https://test.carp.dk");
        assert_eq!(Environment::Dev.url(), "https://dev.carp.dk");

        let addresses: Vec<_> = Environment::ALL.iter().map(|e| e.url()).collect();
        let unique: std::collections::HashSet<_> = addresses.iter().collect();
        assert_eq!(unique.len(), addresses.len(), "two share an address");
    }

    #[test]
    fn environments_are_named_by_their_shorthands() {
        assert_eq!(Environment::parse("prod"), Some(Environment::Production));
        assert_eq!(
            Environment::parse("PRODUCTION"),
            Some(Environment::Production)
        );
        assert_eq!(Environment::parse(" staging "), Some(Environment::Test));
        assert_eq!(Environment::parse("test"), Some(Environment::Test));
        assert_eq!(Environment::parse("development"), Some(Environment::Dev));
        assert_eq!(Environment::parse("prd"), None);
    }

    /// A typo must not quietly fall back to a deployment the user did not
    /// name - least of all to production.
    #[test]
    fn an_unknown_environment_is_refused() {
        let error = resolve_server(None, Some("prod-eu"), None, None)
            .unwrap_err()
            .to_string();
        assert!(error.contains("unknown CARP environment"), "{error}");
        assert!(error.contains("production"), "{error}");
    }

    #[test]
    fn an_address_outranks_a_name_and_a_flag_outranks_the_environment() {
        // --server beats --env.
        assert_eq!(
            resolve_server(Some("https://carp.example.org"), Some("dev"), None, None).unwrap(),
            "https://carp.example.org"
        );
        // --env beats both variables.
        assert_eq!(
            resolve_server(
                None,
                Some("test"),
                Some("https://carp.example.org"),
                Some("dev")
            )
            .unwrap(),
            "https://test.carp.dk"
        );
        // CARP_SERVER beats CARP_ENV.
        assert_eq!(
            resolve_server(None, None, Some("https://carp.example.org"), Some("dev")).unwrap(),
            "https://carp.example.org"
        );
        // CARP_ENV alone still selects.
        assert_eq!(
            resolve_server(None, None, None, Some("dev")).unwrap(),
            "https://dev.carp.dk"
        );
    }

    /// Tokens and cache are keyed by host, so moving between deployments
    /// neither reuses a session nor mixes cached studies.
    #[test]
    fn deployments_do_not_share_a_session() {
        let files: Vec<_> = Environment::ALL
            .iter()
            .map(|environment| {
                let config = Config {
                    server: Url::parse(environment.url()).unwrap(),
                    realm: DEFAULT_REALM.to_owned(),
                    client_id: DEFAULT_CLIENT_ID.to_owned(),
                    data_dir: PathBuf::from("/tmp/carp"),
                    download_dir: PathBuf::from("/tmp/carp"),
                    portal_url: None,
                    portal_study_path: DEFAULT_PORTAL_STUDY_PATH.to_owned(),
                    icons: IconSet::default(),
                };
                (config.token_file(), config.db_path())
            })
            .collect();

        let unique: std::collections::HashSet<_> = files.iter().collect();
        assert_eq!(unique.len(), files.len(), "two deployments share state");
    }
}
