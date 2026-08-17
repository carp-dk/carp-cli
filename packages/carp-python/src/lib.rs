// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! CARP from Python.
//!
//! The same `carp-client` the command line uses, exposed as a module, so a
//! study's data reaches a notebook without going through a subprocess and a
//! JSON parse.
//!
//! Three decisions shape the surface:
//!
//! **It blocks.** The client is async; every method here drives it on a runtime
//! the module owns and returns when it is done. Research code is written top to
//! bottom in a notebook cell, and `await` at every call would buy nothing.
//!
//! **It shares the CLI's session.** [`Client`] reads the same token file
//! `carp auth login` writes, keyed by host, so signing in at a terminal signs
//! in the notebook beside it. `login()` runs the same browser flow.
//!
//! **It returns dictionaries.** Not a class per model. CARP adds fields, and a
//! wrapper class would either need changing for each one or silently drop it;
//! a dict carries whatever arrived. Pandas takes them directly.

use std::sync::Arc;

use carp_client::api::endpoints::{data_streams, exports, files, participants, studies};
use carp_client::api::models::CarpUuid;
use carp_client::api::models::{
    DataStreamId, NamespacedId, ParticipantQuery, SummaryExportRequest,
};
use carp_client::config::Settings;
use carp_client::{ApiError, Authenticator, CarpClient, Config, Error, Moment};
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;

pyo3::create_exception!(
    _carp,
    CarpError,
    PyRuntimeError,
    "Something went wrong talking to CARP."
);
pyo3::create_exception!(
    _carp,
    CarpAuthError,
    CarpError,
    "There is no usable session. Call Client.login()."
);
pyo3::create_exception!(
    _carp,
    CarpNotFoundError,
    CarpError,
    "CARP has no such study, deployment, export or file."
);
pyo3::create_exception!(
    _carp,
    CarpForbiddenError,
    CarpError,
    "The signed-in account may not see this."
);

/// Turn a client failure into the Python exception a caller would catch.
///
/// The distinctions are the ones worth branching on: sign in again, ask for
/// something that exists, ask as somebody else, or give up.
fn to_py(error: Error) -> PyErr {
    let message = error.to_string();
    match &error {
        Error::NoSession(_) | Error::Login(_) => CarpAuthError::new_err(message),
        Error::Config(_) => PyValueError::new_err(message),
        Error::Api(api) => match api {
            ApiError::Unauthorized(_) => CarpAuthError::new_err(message),
            ApiError::Forbidden(_) => CarpForbiddenError::new_err(message),
            ApiError::NotFound(_) => CarpNotFoundError::new_err(message),
            _ => CarpError::new_err(message),
        },
        Error::Io { .. } => CarpError::new_err(message),
    }
}

fn api_to_py(error: ApiError) -> PyErr {
    to_py(Error::Api(error))
}

/// Convert anything the client returns into plain Python values.
fn into_py<T: serde::Serialize>(py: Python<'_>, value: &T) -> PyResult<Py<PyAny>> {
    let json = serde_json::to_value(value)
        .map_err(|error| CarpError::new_err(format!("could not read the response: {error}")))?;
    Ok(pythonize::pythonize(py, &json)?.unbind())
}

/// A window bound, given the way a person would write one.
///
/// `"7d"`, `"2026-08-01"`, `"2026-08-01T09:30:00Z"` or `None` for now — the
/// same forms the command line takes, so a script and a notebook can be
/// written against the same documentation.
fn moment(value: Option<&str>, default: &str) -> PyResult<Moment> {
    value
        .unwrap_or(default)
        .parse()
        .map_err(|message: String| PyValueError::new_err(message))
}

/// A connection to one CARP deployment.
#[pyclass(module = "carp._carp")]
pub struct Client {
    runtime: tokio::runtime::Runtime,
    authenticator: Arc<Authenticator>,
    client: CarpClient,
    config: Config,
}

#[pymethods]
impl Client {
    /// Address a deployment.
    ///
    /// `env` names one of `production`, `test` or `dev`; `server` gives an
    /// address for one that is not named. Anything omitted falls back to
    /// `CARP_*` in the environment, then to a `.env` file, then to production —
    /// exactly as the command line resolves it.
    #[new]
    #[pyo3(signature = (env=None, server=None, data_dir=None, download_dir=None))]
    fn new(
        env: Option<String>,
        server: Option<String>,
        data_dir: Option<std::path::PathBuf>,
        download_dir: Option<std::path::PathBuf>,
    ) -> PyResult<Self> {
        let config = Config::load(&Settings {
            server,
            environment: env,
            data_dir,
            download_dir,
            portal: None,
        })
        .map_err(to_py)?;

        let runtime = tokio::runtime::Runtime::new()
            .map_err(|error| CarpError::new_err(format!("could not start a runtime: {error}")))?;
        let authenticator = Arc::new(Authenticator::new(&config).map_err(to_py)?);
        let client = CarpClient::new(&config, Arc::clone(&authenticator)).map_err(to_py)?;

        Ok(Self {
            runtime,
            authenticator,
            client,
            config,
        })
    }

    /// The deployment this client talks to.
    #[getter]
    fn server(&self) -> String {
        self.config.server.as_str().trim_end_matches('/').to_owned()
    }

    fn __repr__(&self) -> String {
        format!("<carp.Client server={}>", self.server())
    }

    /// Whether there is a usable session, and whose.
    fn status<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let (signed_in, account) = self.runtime.block_on(async {
            (
                self.authenticator.has_session().await,
                self.authenticator.account_label().await,
            )
        });

        let status = PyDict::new(py);
        status.set_item("server", self.server())?;
        status.set_item("signedIn", signed_in)?;
        status.set_item("account", account)?;
        Ok(status)
    }

    /// Sign in through the browser and store the session.
    ///
    /// A no-op when there already is one, unless `force`. The authorization URL
    /// is passed to `on_url` if given and printed otherwise — a headless
    /// interpreter has no browser to open, so something has to show the
    /// address.
    #[pyo3(signature = (force=false, on_url=None))]
    fn login(&self, py: Python<'_>, force: bool, on_url: Option<Py<PyAny>>) -> PyResult<()> {
        let announce = |url: &str| {
            let shown = Python::attach(|py| match &on_url {
                Some(callback) => callback.call1(py, (url,)).map(|_| ()),
                None => py
                    .import("builtins")
                    .and_then(|builtins| {
                        builtins.call_method1("print", (format!("Opening {url}"),))
                    })
                    .map(|_| ()),
            });
            // A callback that raises must not abandon a login already under
            // way; the browser is open and the listener is waiting.
            if let Err(error) = shown {
                Python::attach(|py| error.print(py));
            }
        };

        // Release the GIL: the login blocks until the browser comes back, and
        // holding it would freeze every other thread in the interpreter for
        // as long as the person takes to type their password.
        py.detach(|| {
            self.runtime.block_on(async {
                if force {
                    self.authenticator.login(announce).await
                } else {
                    self.authenticator.ensure_session(announce).await
                }
            })
        })
        .map_err(to_py)
    }

    /// Forget the stored session.
    fn logout(&self) -> PyResult<()> {
        self.runtime
            .block_on(self.authenticator.logout())
            .map_err(to_py)
    }

    /// Studies the signed-in account can see.
    fn studies(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let studies = self.call(py, studies::list(&self.client))?;
        into_py(py, &studies)
    }

    /// One study, by id.
    fn study(&self, py: Python<'_>, study_id: &str) -> PyResult<Py<PyAny>> {
        let studies = self.call(py, studies::list(&self.client))?;
        let study = studies
            .into_iter()
            .find(|study| study.study_id.as_str() == study_id)
            .ok_or_else(|| {
                CarpNotFoundError::new_err(format!("no study {study_id} on {}", self.server()))
            })?;
        into_py(py, &study)
    }

    /// A study's participants.
    ///
    /// Every page by default: a study has hundreds, not millions, and paging
    /// by hand is not what a notebook is for. Pass `all=False` for one page.
    #[pyo3(signature = (study_id, search=None, all=true, page=0, size=50))]
    fn participants(
        &self,
        py: Python<'_>,
        study_id: &str,
        search: Option<String>,
        all: bool,
        page: u32,
        size: u32,
    ) -> PyResult<Py<PyAny>> {
        let query = ParticipantQuery {
            page,
            size: size.max(1),
            search,
            ..ParticipantQuery::default()
        };

        let first = self.call(py, participants::query(&self.client, study_id, &query))?;
        if !all {
            return into_py(py, &first.content);
        }

        let pages = first.page_count(query.size);
        let mut everyone = first.content;
        for page in 1..pages {
            let next = self.call(
                py,
                participants::query(
                    &self.client,
                    study_id,
                    &ParticipantQuery {
                        page,
                        ..query.clone()
                    },
                ),
            )?;
            if next.content.is_empty() {
                break;
            }
            everyone.extend(next.content);
        }
        into_py(py, &everyone)
    }

    /// A study's deployments, one per participant group.
    fn deployments(&self, py: Python<'_>, study_id: &str) -> PyResult<Py<PyAny>> {
        let status = self.call(
            py,
            studies::participant_group_status(&self.client, study_id),
        )?;
        into_py(py, &status.groups)
    }

    /// The measurements one data stream holds, one dictionary per measurement.
    ///
    /// This is the flat form: the stream it came from is folded into every row,
    /// so `pandas.DataFrame(rows)` is a table. `data_stream_raw` returns the
    /// server's own nested response instead.
    #[pyo3(signature = (deployment, device, data_type, start=None, end=None))]
    fn data_stream(
        &self,
        py: Python<'_>,
        deployment: &str,
        device: &str,
        data_type: &str,
        start: Option<&str>,
        end: Option<&str>,
    ) -> PyResult<Py<PyAny>> {
        let (stream, from, to) = self.stream(deployment, device, data_type, start, end)?;
        let batch = self.call(
            py,
            data_streams::query_by_time(&self.client, &stream, from, to),
        )?;
        into_py(py, &batch.rows())
    }

    /// The same window, as the server sent it.
    ///
    /// The measurement payload is not described by CARP's OpenAPI document, so
    /// this is the way to see exactly what a deployment returns when the
    /// flattened form looks wrong.
    #[pyo3(signature = (deployment, device, data_type, start=None, end=None))]
    fn data_stream_raw(
        &self,
        py: Python<'_>,
        deployment: &str,
        device: &str,
        data_type: &str,
        start: Option<&str>,
        end: Option<&str>,
    ) -> PyResult<Py<PyAny>> {
        let (stream, from, to) = self.stream(deployment, device, data_type, start, end)?;
        let raw: serde_json::Value = self.call(
            py,
            self.client.post_json_with_query(
                "/api/data-stream-service/query-by-time",
                &stream,
                &[("from", from.to_rfc3339()), ("to", to.to_rfc3339())],
            ),
        )?;
        into_py(py, &raw)
    }

    /// How much a study has collected, by task and day.
    #[pyo3(signature = (study_id, start=None, end=None, deployment=None, participant=None, scope="study", kind="daily"))]
    #[allow(clippy::too_many_arguments, reason = "the endpoint takes this many")]
    fn data_summary(
        &self,
        py: Python<'_>,
        study_id: &str,
        start: Option<&str>,
        end: Option<&str>,
        deployment: Option<String>,
        participant: Option<String>,
        scope: &str,
        kind: &str,
    ) -> PyResult<Py<PyAny>> {
        let (from, to) = self.window(start, end, "30d")?;
        let summary = self.call(
            py,
            data_streams::summary(
                &self.client,
                &data_streams::SummaryQuery {
                    study_id: study_id.to_owned(),
                    deployment_id: deployment,
                    participant_id: participant,
                    scope: scope.to_owned(),
                    kind: kind.to_owned(),
                    from,
                    to,
                },
            ),
        )?;
        into_py(py, &summary)
    }

    /// A study's exports, newest first.
    fn exports(&self, py: Python<'_>, study_id: &str) -> PyResult<Py<PyAny>> {
        let exports = self.call(py, exports::list(&self.client, study_id))?;
        into_py(py, &exports)
    }

    /// Ask the server to build an export. It is packaged in the background.
    #[pyo3(signature = (study_id, deployments=None, active_only=false))]
    fn create_export(
        &self,
        py: Python<'_>,
        study_id: &str,
        deployments: Option<Vec<String>>,
        active_only: bool,
    ) -> PyResult<()> {
        let request = SummaryExportRequest {
            deployment_ids: deployments
                .filter(|ids| !ids.is_empty())
                .map(|ids| ids.into_iter().map(CarpUuid::new).collect()),
            active_deployments_only: active_only.then_some(true),
        };
        self.call(
            py,
            exports::request_summary(&self.client, study_id, &request),
        )
    }

    /// Download a finished export. Returns the path it was written to.
    #[pyo3(signature = (study_id, export_id, to=None))]
    fn download_export(
        &self,
        py: Python<'_>,
        study_id: &str,
        export_id: &str,
        to: Option<std::path::PathBuf>,
    ) -> PyResult<String> {
        let export = self
            .call(py, exports::list(&self.client, study_id))?
            .into_iter()
            .find(|export| export.id == export_id)
            .ok_or_else(|| {
                CarpNotFoundError::new_err(format!("no export {export_id} in study {study_id}"))
            })?;
        if !export.status.is_downloadable() {
            return Err(CarpError::new_err(format!(
                "export {export_id} is {} - only a finished export can be downloaded",
                export.status.label()
            )));
        }

        self.fetch(
            py,
            &exports::download_path(study_id, export_id),
            &export.display_name(),
            to,
        )
    }

    /// A study's uploaded files.
    #[pyo3(signature = (study_id, query=None))]
    fn files(&self, py: Python<'_>, study_id: &str, query: Option<&str>) -> PyResult<Py<PyAny>> {
        let files = self.call(py, files::list(&self.client, study_id, query))?;
        into_py(py, &files)
    }

    /// Download one. Returns the path it was written to.
    #[pyo3(signature = (study_id, file_id, to=None))]
    fn download_file(
        &self,
        py: Python<'_>,
        study_id: &str,
        file_id: i32,
        to: Option<std::path::PathBuf>,
    ) -> PyResult<String> {
        let file = self
            .call(py, files::list(&self.client, study_id, None))?
            .into_iter()
            .find(|file| file.id == file_id)
            .ok_or_else(|| {
                CarpNotFoundError::new_err(format!("no file {file_id} in study {study_id}"))
            })?;

        self.fetch(
            py,
            &files::download_path(study_id, file_id),
            file.download_name(),
            to,
        )
    }
}

/// The parts that are not exposed to Python.
impl Client {
    /// Drive one request to completion.
    ///
    /// Releases the GIL while it waits: a CARP request is network-bound, and
    /// holding the lock would stop every other Python thread for its duration.
    fn call<T: Send>(
        &self,
        py: Python<'_>,
        future: impl Future<Output = Result<T, ApiError>> + Send,
    ) -> PyResult<T> {
        py.detach(|| self.runtime.block_on(future))
            .map_err(api_to_py)
    }

    fn window(
        &self,
        start: Option<&str>,
        end: Option<&str>,
        default_start: &str,
    ) -> PyResult<(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)> {
        let from = moment(start, default_start)?.resolve();
        let to = moment(end, "now")?.resolve();
        if from > to {
            return Err(PyValueError::new_err(format!(
                "the window ends before it starts: {} is after {}",
                from.to_rfc3339(),
                to.to_rfc3339()
            )));
        }
        Ok((from, to))
    }

    fn stream(
        &self,
        deployment: &str,
        device: &str,
        data_type: &str,
        start: Option<&str>,
        end: Option<&str>,
    ) -> PyResult<(
        DataStreamId,
        chrono::DateTime<chrono::Utc>,
        chrono::DateTime<chrono::Utc>,
    )> {
        let data_type: NamespacedId = data_type
            .parse()
            .map_err(|message: String| PyValueError::new_err(format!("data_type: {message}")))?;
        let (from, to) = self.window(start, end, "7d")?;
        Ok((DataStreamId::new(deployment, device, data_type), from, to))
    }

    fn fetch(
        &self,
        py: Python<'_>,
        api_path: &str,
        fallback_name: &str,
        to: Option<std::path::PathBuf>,
    ) -> PyResult<String> {
        let directory = to.unwrap_or_else(|| self.config.download_dir.clone());
        let transfer = py
            .detach(|| {
                self.runtime.block_on(carp_client::transfer::download(
                    &self.client,
                    api_path,
                    &directory,
                    fallback_name,
                    |_, _| {},
                ))
            })
            .map_err(api_to_py)?;
        Ok(transfer.path.display().to_string())
    }
}

/// The deployments addressable by name.
#[pyfunction]
fn environments() -> Vec<&'static str> {
    carp_client::Environment::ALL
        .iter()
        .map(|environment| environment.name())
        .collect()
}

#[pymodule]
fn _carp(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<Client>()?;
    module.add_function(wrap_pyfunction!(environments, module)?)?;
    module.add("__version__", env!("CARGO_PKG_VERSION"))?;

    let py = module.py();
    module.add("CarpError", py.get_type::<CarpError>())?;
    module.add("CarpAuthError", py.get_type::<CarpAuthError>())?;
    module.add("CarpNotFoundError", py.get_type::<CarpNotFoundError>())?;
    module.add("CarpForbiddenError", py.get_type::<CarpForbiddenError>())?;
    Ok(())
}
