// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Local persistence (Turso/SQLite): an offline cache of studies and
//! participants plus a log of completed downloads.

pub mod cache;
pub mod schema;

pub use cache::{Cache, DownloadRecord};
