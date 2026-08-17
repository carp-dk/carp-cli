// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Thin HTTP layer over the CARP web service.
//!
//! The client is cheap to clone (everything is behind an `Arc`) so background
//! tasks can each own one. Every request carries a bearer token that the
//! [`Authenticator`] keeps fresh.

use std::sync::Arc;
use std::time::Duration;

use reqwest::{Method, RequestBuilder, Response, StatusCode};
use serde::Serialize;
use serde::de::DeserializeOwned;
use url::Url;

use crate::api::error::{ApiError, ApiResult};
use crate::auth::Authenticator;
use crate::config::Config;
use crate::error::{Error, Result};

/// Connection setup budget. Deliberately no overall request timeout: exports
/// can take a while to stream.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
/// Give up when a response stalls for this long.
const READ_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Clone)]
pub struct CarpClient {
    inner: Arc<Inner>,
}

struct Inner {
    http: reqwest::Client,
    base: Url,
    auth: Arc<Authenticator>,
}

impl CarpClient {
    pub fn new(config: &Config, auth: Arc<Authenticator>) -> Result<Self> {
        let http = reqwest::ClientBuilder::new()
            .user_agent(concat!("carp-cli/", env!("CARGO_PKG_VERSION")))
            .connect_timeout(CONNECT_TIMEOUT)
            .read_timeout(READ_TIMEOUT)
            .build()
            .map_err(|error| Error::config(format!("building the CARP HTTP client: {error}")))?;

        Ok(Self {
            inner: Arc::new(Inner {
                http,
                base: config.server.clone(),
                auth,
            }),
        })
    }

    pub fn server(&self) -> &str {
        self.inner.base.as_str()
    }

    /// `GET path` decoded as JSON.
    pub async fn get_json<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> ApiResult<T> {
        let request = self.request(Method::GET, path).await?.query(query);
        let response = self.send(request).await?;
        decode(response).await
    }

    /// `GET path` as text. Used by the endpoints that return a bare string.
    pub async fn get_text(&self, path: &str) -> ApiResult<String> {
        let request = self.request(Method::GET, path).await?;
        let response = self.send(request).await?;
        let body = response.text().await?;
        // A JSON string body arrives quoted; show it unquoted.
        Ok(serde_json::from_str::<String>(&body).unwrap_or(body))
    }

    /// `POST path` with a JSON body, decoding the JSON response.
    pub async fn post_json<B: Serialize + ?Sized, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> ApiResult<T> {
        let request = self.request(Method::POST, path).await?.json(body);
        let response = self.send(request).await?;
        decode(response).await
    }

    /// `POST path?query` with a JSON body, decoding the JSON response.
    ///
    /// Both at once is unusual, but `query-by-time` takes its window in the
    /// query string and the stream it applies to in the body.
    pub async fn post_json_with_query<B: Serialize + ?Sized, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
        query: &[(&str, String)],
    ) -> ApiResult<T> {
        let request = self
            .request(Method::POST, path)
            .await?
            .query(query)
            .json(body);
        let response = self.send(request).await?;
        decode(response).await
    }

    /// `POST path` with a JSON body, ignoring the response body.
    pub async fn post_ok<B: Serialize + ?Sized>(&self, path: &str, body: &B) -> ApiResult<()> {
        let request = self.request(Method::POST, path).await?.json(body);
        self.send(request).await?;
        Ok(())
    }

    /// `DELETE path`, ignoring the response body.
    pub async fn delete_ok(&self, path: &str) -> ApiResult<()> {
        let request = self.request(Method::DELETE, path).await?;
        self.send(request).await?;
        Ok(())
    }

    /// `GET path` returning the raw response, for streaming downloads.
    pub async fn get_stream(&self, path: &str) -> ApiResult<Response> {
        let request = self.request(Method::GET, path).await?;
        self.send(request).await
    }

    /// Build an authorised request for `path`.
    async fn request(&self, method: Method, path: &str) -> ApiResult<RequestBuilder> {
        let token = self
            .inner
            .auth
            .access_token()
            .await
            .map_err(|error| ApiError::Unauthorized(format!("{error}")))?;
        Ok(self
            .inner
            .http
            .request(method, self.url(path))
            .bearer_auth(token))
    }

    /// Send a request and turn any non-success status into an [`ApiError`].
    async fn send(&self, request: RequestBuilder) -> ApiResult<Response> {
        let response = request.send().await?;
        let status = response.status();
        if status.is_success() || status == StatusCode::NO_CONTENT {
            return Ok(response);
        }
        let body = response.text().await.unwrap_or_default();
        Err(ApiError::from_status(status.as_u16(), &body))
    }

    fn url(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.inner.base.as_str().trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }
}

/// Decode a JSON body, reporting the offending payload when it does not match.
async fn decode<T: DeserializeOwned>(response: Response) -> ApiResult<T> {
    let body = response.text().await?;
    serde_json::from_str(&body).map_err(|error| {
        ApiError::Decode(format!(
            "{error} (body: {})",
            body.chars().take(160).collect::<String>()
        ))
    })
}
