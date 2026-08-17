// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Typed views of the CARP web service payloads described in `api-docs.json`.
//!
//! Every struct is `#[serde(default)]` so a field the deployment does not send
//! (or one added later) cannot fail a whole response.

pub mod account;
pub mod common;
pub mod data_stream;
pub mod deployment;
pub mod export;
pub mod file;
pub mod participant;
pub mod protocol;
pub mod study;

// Convenience prelude: the model names are re-exported as a set, even the ones
// no screen happens to name today.
#[allow(unused_imports, reason = "shared model prelude")]
mod prelude {
    pub use super::account::{Account, AccountRole};
    pub use super::common::{CarpInstant, CarpUuid, format_bytes, format_instant};
    pub use super::data_stream::{
        DataPointCount, DataStreamBatch, DataStreamId, DataStreamSequence, DataStreamSummary,
        DeploymentStatistic, DeploymentStatistics, Measurement, MeasurementRow, NamespacedId,
    };
    pub use super::deployment::{
        DeploymentStatus, DeviceInfo, DeviceStatus, ParticipantGroup, ParticipantGroupStatus,
    };
    pub use super::export::{Export, ExportKind, ExportStatus, SummaryExportRequest};
    pub use super::file::StudyFile;
    pub use super::participant::{
        DEFAULT_PAGE_SIZE, ParticipantPage, ParticipantQuery, ParticipantSortBy,
        ParticipantSummary, SortDirection,
    };
    pub use super::protocol::{ApiVersion, ProtocolOverview, ProtocolRequest, ProtocolVersion};
    pub use super::study::{InactiveDeployment, StudyOverview};
}

pub use prelude::*;
