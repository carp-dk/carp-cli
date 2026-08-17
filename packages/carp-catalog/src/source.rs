// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Fetching the upstream protocols over the GitHub API.
//!
//! Three requests, in order:
//!
//! 1. `GET /repos/{owner}/{repo}/commits/{ref}` - which commit `main` is at.
//!    Everything after is pinned to that SHA, so a push landing mid-sync
//!    cannot produce a snapshot that is half one commit and half another.
//! 2. `GET /repos/{owner}/{repo}/git/trees/{sha}?recursive=1` - the file list,
//!    filtered to the paths that matter.
//! 3. `raw.githubusercontent.com/{owner}/{repo}/{sha}/{path}` - the documents.
//!
//! Only `*/carp/resources/protocol.json` is fetched, not a clone or a tarball:
//! the repository is a Flutter workspace whose checkout is tens of megabytes,
//! of which the ten protocol documents are a fraction of a percent.
//!
//! No authentication is used. The repository is public, and unauthenticated
//! GitHub allows 60 requests an hour per address - a sync costs about twelve.
//! A token is honoured when [`GitHubSource::with_token`] is given one, which
//! matters on shared networks where the limit is reached by other people.

use serde::Deserialize;

use crate::{Error, Result, UPSTREAM_OWNER, UPSTREAM_REPO};

/// Where in a study's directory its protocol lives.
const PROTOCOL_SUFFIX: &str = "/carp/resources/protocol.json";

/// A commit of the upstream repository: the thing a catalogue is versioned by.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, Deserialize)]
pub struct Commit {
    /// Full 40-character SHA.
    pub sha: String,
    /// Author date, as an ISO-8601 instant.
    pub date: String,
    /// First line of the commit message.
    pub subject: String,
}

impl Commit {
    /// The seven-character form used in the interface.
    pub fn short_sha(&self) -> &str {
        let end = self
            .sha
            .char_indices()
            .nth(7)
            .map_or(self.sha.len(), |(index, _)| index);
        &self.sha[..end]
    }
}

/// One study's protocol, as fetched.
#[derive(Debug, Clone, PartialEq, serde::Serialize, Deserialize)]
pub struct ProtocolDocument {
    /// Directory the protocol came from, which is the study's name upstream:
    /// `neuropathy`, `catch`, `demo`.
    pub study: String,
    /// Path within the repository, kept so a snapshot can say where a document
    /// came from.
    pub path: String,
    /// The document itself, verbatim.
    pub json: String,
}

/// Reads the upstream repository over the GitHub API.
#[derive(Debug, Clone)]
pub struct GitHubSource {
    http: reqwest::Client,
    owner: String,
    repo: String,
    token: Option<String>,
}

impl GitHubSource {
    /// A source reading `carp-dk/carp_study_app_configurations`.
    pub fn new() -> Result<Self> {
        Self::for_repository(UPSTREAM_OWNER, UPSTREAM_REPO)
    }

    /// A source reading another repository, for testing and for forks.
    pub fn for_repository(owner: &str, repo: &str) -> Result<Self> {
        let http = reqwest::Client::builder()
            .user_agent(concat!("carp-cli/", env!("CARGO_PKG_VERSION")))
            .timeout(std::time::Duration::from_secs(60))
            .build()?;
        Ok(Self {
            http,
            owner: owner.to_owned(),
            repo: repo.to_owned(),
            token: None,
        })
    }

    /// Authenticate, raising the API rate limit from 60 requests an hour to
    /// 5000. Only needed on a shared address.
    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// The commit `reference` currently points at.
    ///
    /// `reference` may be a branch, a tag or a SHA; [`crate::UPSTREAM_BRANCH`] is
    /// what a plain sync uses.
    pub async fn head(&self, reference: &str) -> Result<Commit> {
        #[derive(Deserialize)]
        struct Response {
            sha: String,
            commit: CommitDetail,
        }
        #[derive(Deserialize)]
        struct CommitDetail {
            message: String,
            author: Author,
        }
        #[derive(Deserialize)]
        struct Author {
            date: String,
        }

        let url = format!(
            "https://api.github.com/repos/{}/{}/commits/{reference}",
            self.owner, self.repo
        );
        let response: Response = self.get(&url).await?;

        Ok(Commit {
            sha: response.sha,
            date: response.commit.author.date,
            subject: response
                .commit
                .message
                .lines()
                .next()
                .unwrap_or_default()
                .to_owned(),
        })
    }

    /// Paths of every study protocol at `sha`, in a stable order.
    pub async fn protocol_paths(&self, sha: &str) -> Result<Vec<String>> {
        #[derive(Deserialize)]
        struct Tree {
            tree: Vec<Entry>,
            #[serde(default)]
            truncated: bool,
        }
        #[derive(Deserialize)]
        struct Entry {
            path: String,
            r#type: String,
        }

        let url = format!(
            "https://api.github.com/repos/{}/{}/git/trees/{sha}?recursive=1",
            self.owner, self.repo
        );
        let tree: Tree = self.get(&url).await?;

        // GitHub truncates very large trees. Silently syncing a partial
        // catalogue would be worse than failing, because the missing studies
        // would look like studies that do not exist.
        if tree.truncated {
            return Err(Error::Unexpected(format!(
                "the file listing for {sha} was truncated by GitHub, so the \
                 catalogue would be incomplete"
            )));
        }

        let mut paths: Vec<String> = tree
            .tree
            .into_iter()
            .filter(|entry| entry.r#type == "blob" && entry.path.ends_with(PROTOCOL_SUFFIX))
            .map(|entry| entry.path)
            .collect();
        paths.sort();
        Ok(paths)
    }

    /// Fetch one document at `sha`.
    pub async fn document(&self, sha: &str, path: &str) -> Result<ProtocolDocument> {
        let url = format!(
            "https://raw.githubusercontent.com/{}/{}/{sha}/{path}",
            self.owner, self.repo
        );
        let response = self.request(&url).send().await?;
        let response = check(response, &url)?;
        let json = response.text().await?;

        Ok(ProtocolDocument {
            study: study_name(path),
            path: path.to_owned(),
            json,
        })
    }

    /// Fetch every study protocol at `sha`.
    ///
    /// Documents are fetched one at a time rather than concurrently: ten
    /// small files take about a second in sequence, and a burst of parallel
    /// requests is what trips GitHub's rate limiting for no useful gain.
    pub async fn documents(&self, sha: &str) -> Result<Vec<ProtocolDocument>> {
        let paths = self.protocol_paths(sha).await?;
        if paths.is_empty() {
            return Err(Error::Unexpected(format!(
                "no study protocols found at {sha}; expected files matching \
                 */carp/resources/protocol.json"
            )));
        }

        let mut documents = Vec::with_capacity(paths.len());
        for path in paths {
            documents.push(self.document(sha, &path).await?);
        }
        Ok(documents)
    }

    /// A GET carrying the headers GitHub wants, decoded as JSON.
    async fn get<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T> {
        let response = self
            .request(url)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await?;
        let response = check(response, url)?;
        let body = response.text().await?;
        serde_json::from_str(&body).map_err(|error| {
            Error::Unexpected(format!("{url} did not answer as expected: {error}"))
        })
    }

    fn request(&self, url: &str) -> reqwest::RequestBuilder {
        let request = self.http.get(url);
        match &self.token {
            Some(token) => request.header("Authorization", format!("Bearer {token}")),
            None => request,
        }
    }
}

/// Turn an HTTP failure into an error that says what to do about it.
fn check(response: reqwest::Response, url: &str) -> Result<reqwest::Response> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }

    // The rate limit is the failure a user is most likely to hit, and the
    // least self-explanatory, so it gets its own message.
    let remaining = response
        .headers()
        .get("x-ratelimit-remaining")
        .and_then(|value| value.to_str().ok());
    if status == reqwest::StatusCode::FORBIDDEN && remaining == Some("0") {
        return Err(Error::Unexpected(
            "GitHub's rate limit for this address is exhausted; it resets \
             within the hour, or set GITHUB_TOKEN to raise it"
                .to_owned(),
        ));
    }
    // GitHub answers 404 rather than 403 for a repository the caller cannot
    // see, so as not to confirm that it exists. The upstream configurations
    // repository is private, which makes this the failure an unauthenticated
    // sync hits first - and "does not exist" would send someone looking for
    // the wrong problem.
    if status == reqwest::StatusCode::NOT_FOUND {
        return Err(Error::Unexpected(format!(
            "{url} is not reachable. The {UPSTREAM_OWNER}/{UPSTREAM_REPO} \
             repository is private, so set GITHUB_TOKEN to a token with access \
             to it - `export GITHUB_TOKEN=$(gh auth token)` if you use the \
             GitHub CLI"
        )));
    }
    Err(Error::Unexpected(format!("{url} answered {status}")))
}

/// The study a protocol path belongs to: the first path segment.
fn study_name(path: &str) -> String {
    path.split('/').next().unwrap_or(path).to_owned()
}

#[cfg(test)]
mod tests;
