// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Local cache schema.
//!
//! Records are stored as their JSON payload keyed by id: the API models are
//! the single source of truth for the shape, and the cache never needs a
//! migration when a field is added.

pub const MIGRATIONS: &str = "\
CREATE TABLE IF NOT EXISTS studies (
    study_id  TEXT PRIMARY KEY,
    name      TEXT NOT NULL,
    payload   TEXT NOT NULL,
    cached_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS participants (
    id        TEXT PRIMARY KEY,
    study_id  TEXT NOT NULL,
    payload   TEXT NOT NULL,
    cached_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS downloads (
    id          INTEGER PRIMARY KEY,
    study_id    TEXT,
    label       TEXT NOT NULL,
    path        TEXT NOT NULL,
    bytes       INTEGER NOT NULL,
    finished_at TEXT NOT NULL
);
";
