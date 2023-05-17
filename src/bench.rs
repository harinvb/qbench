use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use async_std::prelude::StreamExt;
use futures::stream::FuturesUnordered;
use glob::glob_with;
use sqlx::{Any, AnyPool, Pool, query};
use sqlx::any::AnyPoolOptions;

use crate::{BenchFailureResult, BenchSuccessResult, QueryBench, QueryBenchParser, QueryBenchResult, QueryBenchResults, QueryRevision, QueryRevisionResult};
use crate::args::Args;
use crate::toml::TomlParser;

/// Run a benchmark with the given arguments.
pub async fn run_bench(args: &Args) -> Result<QueryBenchResults> {
    // Get all files that match the pattern
    let files: Vec<PathBuf> = get_files_matching_pattern(args).await?;
    // Create a parser instance to parse TOML configuration files
    let parser = Arc::new(RefCell::from(TomlParser::new()));
    // Create tasks to parse each file in parallel
    let mut file_parsing_tasks = FuturesUnordered::new();
    for file in files {
        let parser = parser.clone();
        file_parsing_tasks.push(async move {
            parser.borrow().parse(&file).await
        });
    }
    // Collect the query benchmarks from the parsed files
    let mut query_benches = vec![];
    while let Some(query_bench) = file_parsing_tasks.next().await {
        match query_bench {
            Ok(mut query_bench) => query_benches.append(&mut query_bench.queries),
            Err(e) => println!("{}", e), // Print error message if there was an error parsing a file
        }
    }
    // Create a database connection pool
    let pool = AnyPoolOptions::new().max_connections(args.max_connections).connect(&args.url).await?;
    // Create tasks to run each query benchmark in parallel
    let mut query_bench_tasks = FuturesUnordered::new();
    for bench in query_benches {
        let pool = pool.clone();
        query_bench_tasks.push(async move {
            run_query_bench(&bench, &pool, args).await
        });
    }
    // Collect the results of all query benchmarks
    let mut results = vec![];
    while let Some(result) = query_bench_tasks.next().await {
        match result {
            Ok(result) => results.push(result),
            Err(e) => println!("{}", e), // Print error message if there was an error running a query benchmark
        }
    }

    // Return the query benchmark results
    Ok(QueryBenchResults { results })
}

/// Asynchronously finds files matching a pattern in a directory using glob.
///
/// # Arguments
///
/// * `args` - A reference to a struct containing the search directory and the search pattern.
///
/// # Returns
///
/// A `Result` containing a vector of `PathBuf` of files matching the provided pattern.
///
/// # Example
///
/// ```rust
/// use my_crate::args::Args;
/// use my_crate::file_utils::get_files_matching_pattern;
/// use qbench::args::Args;
/// use clap::Parser;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let args = Args::parse()?;
///     let files = get_files_matching_pattern(&args).await?;
///     println!("{:?}", files);
///     Ok(())
/// }
/// ```
async fn get_files_matching_pattern(args: &Args) -> Result<Vec<PathBuf>> {
    // Create glob match options, making it case-insensitive by default.
    let glob_options = glob::MatchOptions {
        case_sensitive: false,
        ..Default::default()
    };

    // Use glob to find files matching the provided pattern in the provided directory.
    Ok(glob_with(
        // Format the search directory and pattern into a single string and convert it to a `&str`.
        format!("{}/{}", args.dir.to_str().unwrap_or("./"), args.pattern).as_ref(),
        glob_options, // Pass the glob match options.
    )?
        .flatten() // Flatten the glob iterator into a vector of `Result`s.
        .filter(|f| f.is_file()) // Filter out directories and other non-files.
        .collect()) // Collect the remaining `PathBuf`s into a vector and return it as a `Result`.
}


/// Runs a benchmark on multiple revisions of a query.
/// Returns the results of each revision as a `QueryBenchResult`.
///
/// # Arguments
///
/// * `bench` - The query bench to run.
/// * `pool` - The database connection pool to use.
/// * `args` - Additional arguments for the benchmark.
///
/// # Returns
///
/// * `Result<QueryBenchResult>` - A tuple containing the benchmark name and its results.
async fn run_query_bench(bench: &QueryBench, pool: &AnyPool, args: &Args) -> Result<QueryBenchResult> {
    // Create a new unordered collection of futures for the sub-benchmarks.
    let mut sub_bench_tasks = FuturesUnordered::new();
    // For each revision in the query bench...
    for revision in &bench.revisions {
        // Clone the connection pool.
        let pool = pool.clone();
        // Push a future onto the task list that runs the sub-benchmark.
        sub_bench_tasks.push(async move {
            run_revision_bench(revision, &pool, args).await
        });
    }
    // Collect the results of each sub-benchmark.
    let mut results = vec![];
    while let Some(result) = sub_bench_tasks.next().await {
        match result {
            // If the sub-benchmark was successful, add it to the list of results.
            Ok(result) => results.push(result),
            // If an error occurred, print it and continue with the next sub-benchmark.
            Err(e) => println!("{}", e),
        }
    }
    // Return the name of the query bench and its results.
    Ok(QueryBenchResult { name: bench.name.clone(), revision_result: results })
}


/// Benchmark a query revision using the provided arguments.
///
/// # Arguments
///
/// * `query_revision` - The `QueryRevision` to be benchmarked.
/// * `pool` - An instance of `any_pool::Pool` for executing queries.
/// * `args` - An instance of `Args` containing the benchmarking arguments.
///
/// # Returns
///
/// A Result containing a QueryRevisionResult indicating the success or failure of the benchmark operation.
async fn run_revision_bench(query_revision: &QueryRevision, pool: &Pool<Any>, args: &Args) -> Result<QueryRevisionResult> {
    let mut tx = pool.begin().await?; // begin a transaction

    let mut bench_success_res = BenchSuccessResult { // initialize a BenchSuccessResult with the default value and the name of the query revision
        revision_name: query_revision.name.clone(), // cloning the name to avoid borrowing issues
        ..Default::default()
    };

    if let Some(pre_script) = &query_revision.pre_script { // if a pre_script is defined
        let pre_script_start = Instant::now(); // note the time before executing the script
        let pre_script_result = query(pre_script).execute(&mut tx).await; // execute the pre_script on the transaction
        let pre_script_duration = pre_script_start.elapsed(); // calculate the duration of the pre_script execution
        match pre_script_result { // check the result of the pre_script execution
            Ok(_) => bench_success_res.pre_script_duration = pre_script_duration, // if successful, record the duration in the BenchSuccessResult
            Err(e) => return Ok(QueryRevisionResult::PreScriptFailure(BenchFailureResult { // if failed, return an error wrapped in a QueryRevisionResult indicating pre_script failure
                revision_name: query_revision.name.clone(),
                error: e.to_string(),
            })),
        }
    }

    let mut durations = vec![]; // initialize a vector of Duration to record the duration of each execution of the query
    for _ in 0..args.iterations { // iterate over the number of iterations specified in the Args
        let start = Instant::now(); // take note of the time before executing the query
        let query_result = query(query_revision.query.as_str()).execute(&mut tx).await; // execute the query on the transaction
        match query_result {
            Ok(_) => {
                durations.push(start.elapsed()); // if successful, record the duration in the vector
            }
            Err(e) => {
                return Ok(QueryRevisionResult::Failure(BenchFailureResult { // if failed, return an error wrapped in a QueryRevisionResult indicating query failure
                    revision_name: query_revision.name.clone(),
                    error: e.to_string(),
                }));
            }
        };
    }

    bench_success_res.durations = durations; // record the vector of durations in the BenchSuccessResult

    let total = bench_success_res.durations.len() as f64; // calculate the total number of executions
    bench_success_res.avg_duration = bench_success_res.durations.iter().sum::<Duration>().div_f64(total); // calculate the average duration and add it to the BenchSuccessResult

    if let Some(post_script) = &query_revision.post_script { // if a post_script is defined
        let post_script_start = Instant::now(); // note the time before executing the script
        let post_script_result = query(post_script).execute(&mut tx).await; // execute the post_script on the transaction
        let post_script_duration = post_script_start.elapsed(); // calculate the duration of the post_script execution
        match post_script_result { // check the result of the post_script execution
            Ok(_) => bench_success_res.post_script_duration = post_script_duration, // if successful, record the duration in the BenchSuccessResult
            Err(e) => return Ok(QueryRevisionResult::PostScriptFailure(BenchFailureResult { // if failed, return an error wrapped in a QueryRevisionResult indicating post_script failure
                revision_name: query_revision.name.clone(),
                error: e.to_string(),
            })),
        }
    }

    tx.rollback().await?; // rollback the transaction

    Ok(QueryRevisionResult::Success(bench_success_res)) // return a QueryRevisionResult indicating success and containing the BenchSuccessResult
}
