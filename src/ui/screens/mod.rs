// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! One module per screen. Screens render state and never mutate it, apart
//! from the selection state ratatui keeps for tables.
//!
//! Every screen is laid out the same way: the list of things on the left, and
//! a panel describing the highlighted one on the right.

pub mod downloads;
pub mod participant;
pub mod studies;
pub mod studio;
pub mod study;
