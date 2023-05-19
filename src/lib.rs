use std::path::Path;

use anyhow::Result;
use humantime::format_duration;
use serde::{Deserialize, Serialize};
use tabled::{Table, Tabled};

use tabled::settings::Style;

use tokio::time::Duration;

pub mod args;
pub mod bench;
pub mod toml;
pub(crate) mod util;

// Define a struct to hold a single benchmark result, including revision-specific results.
#[derive(Serialize, Deserialize, Debug, Clone, Default, Tabled)]
#[tabled(rename_all = "PascalCase")]
pub struct QueryBenchResult {
    pub name: String,
    #[tabled(display_with = "format_rev_result")]
    pub results: Vec<QueryRevisionResult>,
}

// Define an enum to represent different types of query revision results.
#[derive(Serialize, Deserialize, Debug, Clone, Default, Tabled)]
#[tabled(rename_all = "PascalCase")]
pub struct QueryRevisionResult {
    pub revision_name: String,
    #[tabled(skip)]
    pub durations: Vec<Duration>,
    #[tabled(display_with = "duration_format")]
    pub avg_query_duration: Duration,
    #[tabled(display_with = "duration_format")]
    pub pre_script_duration: Duration,
    #[tabled(display_with = "duration_format")]
    pub post_script_duration: Duration,
}

fn duration_format(duration: &Duration) -> String {
    format_duration(*duration).to_string()
}

fn format_rev_result(rev_result: &Vec<QueryRevisionResult>) -> String {
    let mut table = Table::new(rev_result);
    table.with(Style::modern()).to_string()
}

// Define a struct to hold multiple QueryBench instances.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct QueryBenches {
    pub queries: Vec<QueryBench>,
}

// Define a struct to hold a single query benchmark, including multiple revisions.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct QueryBench {
    pub name: String,
    pub revisions: Vec<QueryRevision>,
}

// Define a struct to hold the details of a single query revision benchmark.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
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
