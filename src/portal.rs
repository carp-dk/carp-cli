// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Handing a study over to the CARP web portal in a browser.
//!
//! There is no login step to perform. The CLI authenticates through the
//! system browser, so Keycloak has already set its single-sign-on cookie for
//! the realm there; opening a portal address in that same browser signs the
//! user in silently. Only if the browser's Keycloak session has lapsed does
//! the portal show a login page.
//!
//! The portal's address is not in the OpenAPI document, so it is resolved in
//! this order:
//!
//! 1. `--portal` / `CARP_PORTAL_URL`, when the deployment is known.
//! 2. The OAuth redirect URIs the server has registered, which are the
//!    deployment's own record of where its web clients live.
//! 3. The API server itself.

use std::collections::HashMap;

use url::Url;

use carp_client::config::Config;

/// Placeholder replaced with the study id in the study path template.
const STUDY_PLACEHOLDER: &str = "{study}";

#[derive(Debug, Clone)]
pub struct Portal {
    /// Set from configuration, or discovered from the server.
    base: Option<Url>,
    /// Used until something better is known.
    fallback: Url,
    /// Path template containing [`STUDY_PLACEHOLDER`].
    study_path: String,
    /// True when the base came from configuration and must not be replaced.
    pinned: bool,
}

impl Portal {
    pub fn new(config: &Config) -> Self {
        Self {
            base: config.portal_url.clone(),
            fallback: config.server.clone(),
            study_path: config.portal_study_path.clone(),
            pinned: config.portal_url.is_some(),
        }
    }

    /// Where the portal is believed to live right now.
    pub fn base(&self) -> &Url {
        self.base.as_ref().unwrap_or(&self.fallback)
    }

    /// True once the address came from configuration or the server, rather
    /// than being assumed.
    pub fn is_resolved(&self) -> bool {
        self.base.is_some()
    }

    /// Adopt an address discovered from the server, unless one was configured.
    pub fn discovered(&mut self, base: Url) {
        if !self.pinned {
            self.base = Some(base);
        }
    }

    /// The browser address for one study.
    pub fn study_url(&self, study_id: &str) -> Url {
        let path = self.study_path.replace(STUDY_PLACEHOLDER, study_id);
        let base = self.base().as_str().trim_end_matches('/').to_owned();
        Url::parse(&format!("{base}/{}", path.trim_start_matches('/')))
            .unwrap_or_else(|_| self.base().clone())
    }
}

/// Pick the portal origin out of the deployment's registered redirect URIs.
///
/// Loopback addresses belong to CLIs like this one, so they are skipped; what
/// remains is a hosted web client.
pub fn origin_from_redirect_uris(uris: &HashMap<String, Vec<String>>) -> Option<Url> {
    let mut candidates: Vec<Url> = uris
        .values()
        .flatten()
        .filter_map(|uri| Url::parse(uri.trim_end_matches('*')).ok())
        .filter(|url| matches!(url.scheme(), "http" | "https"))
        .filter(|url| !is_loopback(url))
        .collect();

    // Stable choice: the same deployment must always resolve the same way.
    candidates.sort_by(|left, right| left.as_str().cmp(right.as_str()));
    let chosen = candidates.into_iter().next()?;

    let mut origin = chosen.clone();
    origin.set_path("");
    origin.set_query(None);
    origin.set_fragment(None);
    Some(origin)
}

fn is_loopback(url: &Url) -> bool {
    matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> Config {
        Config {
            server: Url::parse("https://dev.carp.dk").unwrap(),
            realm: "Carp".to_owned(),
            client_id: "carp-cli".to_owned(),
            data_dir: std::env::temp_dir(),
            download_dir: std::env::temp_dir(),
            portal_url: None,
            portal_study_path: carp_client::config::DEFAULT_PORTAL_STUDY_PATH.to_owned(),
        }
    }

    #[test]
    fn the_api_server_is_used_until_something_better_is_known() {
        let portal = Portal::new(&config());
        assert!(!portal.is_resolved());
        assert_eq!(
            portal.study_url("abc").as_str(),
            "https://dev.carp.dk/studies/abc"
        );
    }

    #[test]
    fn a_configured_address_outranks_discovery() {
        let mut config = config();
        config.portal_url = Some(Url::parse("https://portal.example.org/app/").unwrap());
        let mut portal = Portal::new(&config);
        portal.discovered(Url::parse("https://discovered.example.org").unwrap());

        assert_eq!(
            portal.study_url("abc").as_str(),
            "https://portal.example.org/app/studies/abc"
        );
    }

    #[test]
    fn the_portal_is_discovered_from_registered_redirect_uris() {
        let uris = HashMap::from([
            (
                "carp-cli".to_owned(),
                vec!["http://127.0.0.1:8080/callback".to_owned()],
            ),
            (
                "carp-webapp".to_owned(),
                vec![
                    "https://portal.carp.dk/oauth2/callback".to_owned(),
                    "https://portal.carp.dk/*".to_owned(),
                ],
            ),
        ]);

        let origin = origin_from_redirect_uris(&uris).expect("a hosted client is registered");
        assert_eq!(origin.as_str(), "https://portal.carp.dk/");

        let mut portal = Portal::new(&config());
        portal.discovered(origin);
        assert!(portal.is_resolved());
        assert_eq!(
            portal.study_url("abc").as_str(),
            "https://portal.carp.dk/studies/abc"
        );
    }

    #[test]
    fn a_loopback_only_deployment_discovers_nothing() {
        let uris = HashMap::from([(
            "carp-cli".to_owned(),
            vec!["http://localhost:1234/".to_owned()],
        )]);
        assert!(origin_from_redirect_uris(&uris).is_none());
    }
}
