use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_with::{DurationNanoSeconds, serde_as};
use tabled::Tabled;
use tokio::time::Duration;

pub mod args;
pub mod bench;
pub mod util;
mod parser;

// Define a struct to hold a single benchmark result, including revision-specific results.
#[derive(Serialize, Debug, Clone, Default, Tabled)]
#[tabled(rename_all = "PascalCase")]
pub struct QueryBenchResult {
    pub name: String,
    #[tabled(display_with = "util::format_rev_result")]
    pub results: Vec<QueryRevisionResult>,
}

// Define an enum to represent different types of query revision results.
#[serde_as]
#[derive(Serialize, Debug, Clone, Default, Tabled)]
#[tabled(rename_all = "PascalCase")]
pub struct QueryRevisionResult {
    pub revision_name: String,

    #[tabled(skip)]
    #[serde_as(as = "Vec<DurationNanoSeconds<u64>>")]
    #[serde(rename = "durations_ns")]
    pub durations: Vec<Duration>,

    #[tabled(display_with = "util::format_duration_pretty")]
    #[serde_as(as = "DurationNanoSeconds<u64>")]
    #[serde(rename = "avg_query_duration_ns")]
    pub avg_query_duration: Duration,

    #[tabled(display_with = "util::format_duration_pretty")]
    #[serde_as(as = "DurationNanoSeconds<u64>")]
    #[serde(rename = "pre_script_duration_ns")]
    pub pre_script_duration: Duration,

    #[tabled(display_with = "util::format_duration_pretty")]
    #[serde_as(as = "DurationNanoSeconds<u64>")]
    #[serde(rename = "post_script_duration_ns")]
    pub post_script_duration: Duration,
}

// Define a struct to hold multiple QueryBench instances.
#[derive(Deserialize, Debug, Clone, Default)]
pub struct QueryBenches {
    pub queries: Vec<QueryBench>,
}

// Define a struct to hold a single query benchmark, including multiple revisions.
#[derive(Deserialize, Debug, Clone, Default)]
pub struct QueryBench {
    pub name: String,
    pub revisions: Vec<QueryRevision>,
}

// Define a struct to hold the details of a single query revision benchmark.
#[derive(Deserialize, Debug, Clone, Default)]
pub struct QueryRevision {
    pub name: String,
    pub query: String,
    pub pre_script: Option<String>,
    pub post_script: Option<String>,
}

// Define a trait for parsing query benchmarks.
#[async_trait::async_trait]
trait QueryBenchParser {
    async fn parse(&self, path: &Path) -> Result<QueryBenches>;
}
