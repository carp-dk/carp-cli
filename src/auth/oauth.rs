// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! OAuth2 authorization-code flow with PKCE against the CARP Keycloak realm.
//!
//! A CLI is a public client, so no secret is embedded: the browser performs the
//! login and redirects to a loopback listener bound to an ephemeral port.

use std::collections::HashMap;
use std::time::Duration;

use color_eyre::Result;
use color_eyre::eyre::{Context, bail, eyre};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, CsrfToken, PkceCodeChallenge, RedirectUrl, RefreshToken,
    Scope, TokenResponse, TokenUrl, basic::BasicClient,
};
use subtle::ConstantTimeEq;
use tiny_http::{Header, Response, Server};
use url::Url;

use crate::auth::token::TokenSet;
use crate::config::Config;

/// How long the user gets to complete the browser login.
const LOGIN_TIMEOUT: Duration = Duration::from_secs(300);

const SCOPES: [&str; 5] = ["openid", "profile", "email", "carp", "offline_access"];

/// Run the interactive login. Blocks until the browser redirect arrives.
pub async fn login(config: &Config, http: &reqwest::Client) -> Result<TokenSet> {
    // Bind before building the authorization URL, so the chosen port is known.
    let callback_server = Server::http("127.0.0.1:0").map_err(|e| eyre!(e.to_string()))?;
    let port = callback_server
        .server_addr()
        .to_ip()
        .ok_or_else(|| eyre!("callback listener is not an IP socket"))?
        .port();
    let redirect_uri = format!("http://127.0.0.1:{port}");

    let client = BasicClient::new(ClientId::new(config.client_id.clone()))
        .set_auth_uri(AuthUrl::new(config.auth_url())?)
        .set_token_uri(TokenUrl::new(config.token_url())?)
        .set_redirect_uri(RedirectUrl::new(redirect_uri)?);

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
    let mut request = client.authorize_url(CsrfToken::new_random);
    for scope in SCOPES {
        request = request.add_scope(Scope::new(scope.to_owned()));
    }
    let (login_url, expected_state) = request.set_pkce_challenge(pkce_challenge).url();

    println!("Opening {login_url}");
    if webbrowser::open(login_url.as_str()).is_err() {
        println!("Could not open a browser. Open the URL above manually.");
    }

    // tiny_http is blocking, so wait for the redirect off the async runtime.
    let params = tokio::task::spawn_blocking(move || wait_for_callback(callback_server))
        .await
        .context("callback listener panicked")??;

    if let Some(error) = params.get("error") {
        let description = params.get("error_description").map_or("", String::as_str);
        bail!("authorization failed: {error}: {description}");
    }

    let state = params
        .get("state")
        .ok_or_else(|| eyre!("callback has no state"))?;
    if expected_state
        .secret()
        .as_bytes()
        .ct_eq(state.as_bytes())
        .unwrap_u8()
        != 1
    {
        bail!("OAuth state mismatch");
    }

    let code = params
        .get("code")
        .ok_or_else(|| eyre!("callback has no authorization code"))?;

    let tokens = client
        .exchange_code(AuthorizationCode::new(code.clone()))
        .set_pkce_verifier(pkce_verifier)
        .request_async(http)
        .await
        .context("exchanging the authorization code for tokens")?;

    Ok(TokenSet::new(
        tokens.access_token().secret().clone(),
        tokens.refresh_token().map(|t| t.secret().clone()),
        tokens.expires_in(),
    ))
}

/// Exchange a refresh token for a fresh access token.
pub async fn refresh(
    config: &Config,
    http: &reqwest::Client,
    refresh_token: &str,
) -> Result<TokenSet> {
    let client = BasicClient::new(ClientId::new(config.client_id.clone()))
        .set_auth_uri(AuthUrl::new(config.auth_url())?)
        .set_token_uri(TokenUrl::new(config.token_url())?);

    let token = RefreshToken::new(refresh_token.to_owned());
    let tokens = client
        .exchange_refresh_token(&token)
        .request_async(http)
        .await
        .context("refreshing the access token")?;

    Ok(TokenSet::new(
        tokens.access_token().secret().clone(),
        tokens
            .refresh_token()
            .map(|t| t.secret().clone())
            // Keycloak may not re-issue a refresh token; keep the current one.
            .or_else(|| Some(refresh_token.to_owned())),
        tokens.expires_in(),
    ))
}

/// Build the HTTP client used for token requests. Redirects are disabled so a
/// misconfigured endpoint cannot leak the authorization code.
pub fn http_client() -> Result<reqwest::Client> {
    reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .context("building the OAuth HTTP client")
}

/// Accept a single redirect, answer the browser, and return its query string.
fn wait_for_callback(server: Server) -> Result<HashMap<String, String>> {
    let request = server
        .recv_timeout(LOGIN_TIMEOUT)?
        .ok_or_else(|| eyre!("timed out waiting for browser authentication"))?;

    let callback = Url::parse(&format!("http://127.0.0.1{}", request.url()))?;
    let params: HashMap<String, String> = callback.query_pairs().into_owned().collect();

    let html = Header::from_bytes("Content-Type", "text/html; charset=utf-8")
        .map_err(|()| eyre!("invalid content-type header"))?;
    let body = if params.contains_key("code") {
        "<h3>Authentication complete.</h3><p>You can close this tab and return to the CLI.</p>"
    } else {
        "<h3>Authentication failed.</h3><p>Return to the CLI for details.</p>"
    };
    let _ = request.respond(Response::from_string(body).with_header(html));

    Ok(params)
}
