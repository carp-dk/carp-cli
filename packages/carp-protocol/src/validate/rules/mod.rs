// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! The individual validation rules. See [`super`] for what the severities mean.
//!
//! One module per part of the protocol, and one function per rule, so the
//! list of what is checked reads as a list. Rules never stop at the first
//! problem: someone fixing a protocol wants the whole list, not one item at
//! a time.

pub mod devices;
pub mod identity;
pub mod people;
pub mod schedule;
pub mod surveys;
pub mod tasks;

pub use devices::{connections, devices};
pub use identity::identity;
pub use people::participants;
pub use schedule::{task_controls, triggers};
pub use surveys::{surveys, unmodelled_types};
pub use tasks::tasks;
