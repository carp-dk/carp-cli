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

use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, CsrfToken, PkceCodeChallenge, RedirectUrl, RefreshToken,
    Scope, TokenResponse, TokenUrl, basic::BasicClient,
};
use subtle::ConstantTimeEq;
use tiny_http::{Header, Response, Server};
use url::Url;

use crate::auth::token::TokenSet;
use crate::config::Config;
use crate::error::{Error, Result};

/// How long the user gets to complete the browser login.
const LOGIN_TIMEOUT: Duration = Duration::from_secs(300);

const SCOPES: [&str; 5] = ["openid", "profile", "email", "carp", "offline_access"];

/// Run the interactive login. Blocks until the browser redirect arrives.
///
/// `on_url` receives the authorization URL before the browser is opened. The
/// library must not print it itself: one caller is a terminal, another is a
/// Python interpreter, and only they know where a message should go.
pub async fn login(
    config: &Config,
    http: &reqwest::Client,
    on_url: impl Fn(&str),
) -> Result<TokenSet> {
    // Bind before building the authorization URL, so the chosen port is known.
    let callback_server = Server::http("127.0.0.1:0").map_err(|error| {
        Error::login(format!(
            "could not listen for the browser redirect: {error}"
        ))
    })?;
    let port = callback_server
        .server_addr()
        .to_ip()
        .ok_or_else(|| Error::login("callback listener is not an IP socket"))?
        .port();
    let redirect_uri = format!("http://127.0.0.1:{port}");

    let url =
        |what: &str, value: String| Error::login(format!("{what} is not a valid URL: {value}"));
    let client = BasicClient::new(ClientId::new(config.client_id.clone()))
        .set_auth_uri(
            AuthUrl::new(config.auth_url())
                .map_err(|_| url("the authorization endpoint", config.auth_url()))?,
        )
        .set_token_uri(
            TokenUrl::new(config.token_url())
                .map_err(|_| url("the token endpoint", config.token_url()))?,
        )
        .set_redirect_uri(
            RedirectUrl::new(redirect_uri.clone())
                .map_err(|_| url("the redirect address", redirect_uri))?,
        );

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
    let mut request = client.authorize_url(CsrfToken::new_random);
    for scope in SCOPES {
        request = request.add_scope(Scope::new(scope.to_owned()));
    }
    let (login_url, expected_state) = request.set_pkce_challenge(pkce_challenge).url();

    on_url(login_url.as_str());
    let _ = webbrowser::open(login_url.as_str());

    // tiny_http is blocking, so wait for the redirect off the async runtime.
    let params = tokio::task::spawn_blocking(move || wait_for_callback(callback_server))
        .await
        .map_err(|_| Error::login("the callback listener panicked"))??;

    if let Some(error) = params.get("error") {
        let description = params.get("error_description").map_or("", String::as_str);
        return Err(Error::login(format!(
            "authorization failed: {error}: {description}"
        )));
    }

    let state = params
        .get("state")
        .ok_or_else(|| Error::login("the callback carried no state"))?;
    if expected_state
        .secret()
        .as_bytes()
        .ct_eq(state.as_bytes())
        .unwrap_u8()
        != 1
    {
        return Err(Error::login("OAuth state mismatch"));
    }

    let code = params
        .get("code")
        .ok_or_else(|| Error::login("the callback carried no authorization code"))?;

    let tokens = client
        .exchange_code(AuthorizationCode::new(code.clone()))
        .set_pkce_verifier(pkce_verifier)
        .request_async(http)
        .await
        .map_err(|error| {
            Error::login(format!(
                "exchanging the authorization code for tokens: {error}"
            ))
        })?;

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
    let url =
        |what: &str, value: String| Error::login(format!("{what} is not a valid URL: {value}"));
    let client = BasicClient::new(ClientId::new(config.client_id.clone()))
        .set_auth_uri(
            AuthUrl::new(config.auth_url())
                .map_err(|_| url("the authorization endpoint", config.auth_url()))?,
        )
        .set_token_uri(
            TokenUrl::new(config.token_url())
                .map_err(|_| url("the token endpoint", config.token_url()))?,
        );

    let token = RefreshToken::new(refresh_token.to_owned());
    let tokens = client
        .exchange_refresh_token(&token)
        .request_async(http)
        .await
        .map_err(|error| Error::login(format!("refreshing the access token: {error}")))?;

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
        .map_err(|error| Error::login(format!("building the OAuth HTTP client: {error}")))
}

/// Accept a single redirect, answer the browser, and return its query string.
fn wait_for_callback(server: Server) -> Result<HashMap<String, String>> {
    let request = server
        .recv_timeout(LOGIN_TIMEOUT)
        .map_err(|error| Error::login(format!("waiting for the browser redirect: {error}")))?
        .ok_or_else(|| Error::login("timed out waiting for browser authentication"))?;

    let callback = Url::parse(&format!("http://127.0.0.1{}", request.url())).map_err(|error| {
        Error::login(format!(
            "the browser redirected to an unparsable address: {error}"
        ))
    })?;
    let params: HashMap<String, String> = callback.query_pairs().into_owned().collect();

    let html = Header::from_bytes("Content-Type", "text/html; charset=utf-8")
        .map_err(|()| Error::login("invalid content-type header"))?;
    let body = if params.contains_key("code") {
        "<h3>Authentication complete.</h3><p>You can close this tab and return to the CLI.</p>"
    } else {
        "<h3>Authentication failed.</h3><p>Return to the CLI for details.</p>"
    };
    let _ = request.respond(Response::from_string(body).with_header(html));

    Ok(params)
}
