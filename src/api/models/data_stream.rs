// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Data stream models (`data-stream-controller`, `study-deployment-controller`).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::api::models::common::CarpInstant;

/// Upload volume for one task on one day.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DataPointCount {
    pub date: Option<CarpInstant>,
    pub task: String,
    pub quantity: i64,
}

/// Response of `GET /api/data-stream-service/summary`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DataStreamSummary {
    pub study_id: String,
    pub deployment_id: Option<String>,
    pub participant_id: Option<String>,
    pub scope: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub from: Option<CarpInstant>,
    pub to: Option<CarpInstant>,
    pub data: Vec<DataPointCount>,
}

impl DataStreamSummary {
    pub fn total(&self) -> i64 {
        self.data.iter().map(|point| point.quantity).sum()
    }
}

/// Per-deployment upload counts from `POST /api/deployment-service/statistics`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DeploymentStatistics {
    /// deployment id -> statistic name -> counts
    pub statistics: BTreeMap<String, BTreeMap<String, DeploymentStatistic>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DeploymentStatistic {
    pub count: i32,
    pub uploads: BTreeMap<String, i32>,
}
