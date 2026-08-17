// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! One module per CARP controller. Each function maps to a single documented
//! operation in `api-docs.json`.
//!
//! The set is deliberately complete per controller, so an operation is here
//! before a caller needs it.

pub mod accounts;
pub mod data_streams;
pub mod exports;
pub mod files;
pub mod participants;
pub mod protocols;
pub mod studies;
