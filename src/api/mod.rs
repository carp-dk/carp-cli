// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! CARP web service client: transport ([`client`]), typed payloads
//! ([`models`]) and one function per documented operation ([`endpoints`]).

pub mod client;
pub mod endpoints;
pub mod error;
pub mod models;

pub use client::CarpClient;
#[allow(unused_imports, reason = "part of the api module's surface")]
pub use error::{ApiError, ApiResult};
