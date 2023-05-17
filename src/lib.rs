use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod args;
pub mod bench;
pub mod toml;

// Define a struct to hold multiple QueryBenchResult instances.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
pub struct QueryBenchResults {
    pub results: Vec<QueryBenchResult>,
}

// Define a struct to hold a single benchmark result, including revision-specific results.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
pub struct QueryBenchResult {
    pub name: String,
    pub revision_result: Vec<QueryRevisionResult>,
}

// Define an enum to represent different types of query revision results.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum QueryRevisionResult {
    Success(BenchSuccessResult),
    Failure(BenchFailureResult),
    PreScriptFailure(BenchFailureResult),
    PostScriptFailure(BenchFailureResult),
}

// Define a struct to hold the results of a successful query revision benchmark.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
pub struct BenchSuccessResult {
    pub revision_name: String,
    pub durations: Vec<Duration>,
    pub pre_script_duration: Duration,
    pub post_script_duration: Duration,
    pub avg_duration: Duration,
}

// Define a struct to hold the results of a failed query revision benchmark.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
pub struct BenchFailureResult {
    pub revision_name: String,
    pub error: String,
}

// Define a struct to hold multiple QueryBench instances.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
pub struct QueryBenches {
    pub queries: Vec<QueryBench>,
}

// Define a struct to hold a single query benchmark, including multiple revisions.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
pub struct QueryBench {
    pub name: String,
    pub revisions: Vec<QueryRevision>,
}

// Define a struct to hold the details of a single query revision benchmark.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
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
