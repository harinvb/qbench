use std::ops::DerefMut;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use clap::Parser;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use glob::glob_with;
use sqlx::{Any, AnyPool, query, Transaction};
use sqlx::any::AnyPoolOptions;
use sqlx::migrate::Migrate;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};

use crate::{QueryBench, QueryBenchParser, QueryBenchResult, QueryRevision, QueryRevisionResult};
use crate::args::Args;
use crate::parser::DefaultParser;
use crate::util::extract_multiline_queries;

#[derive(Debug, Clone)]
pub struct QBench {
    pool: AnyPool,
    pub args: Arc<Args>,
    pub display_progress: bool,
}

impl QBench {
    /// Create a new instance of `Self` struct, which holds a connection pool and `Args` configuration arguments.
    ///
    /// # Arguments
    ///
    /// * `args` - `Args` type representing the application's configuration arguments.
    /// * `display_progress` - `bool` type which determines whether to display progress or not while connecting to the database.
    ///
    /// # Example
    /// ```rust
    /// use clap::Parser;
    /// use qbench::args::Args;
    /// let args = Args::parse();
    /// async {
    ///     let db = Self::new(args, true).await.unwrap();
    /// }
    /// ```
    pub async fn new(args: Args, display_progress: bool) -> Result<Self> {
        //Create a connection pool with maximum connections passed from args and connect to the database.
        let pool = AnyPoolOptions::new()
            .max_connections(args.max_connections)
            .acquire_timeout(Duration::from_secs(args.connection_acquire_timeout))
            .idle_timeout(Duration::from_secs(args.connection_idle_timeout))
            .connect_lazy(&args.url)?;
        //Return a new instance of Self struct.
        Ok(Self {
            pool,
            args: Arc::new(args),
            display_progress,
        })
    }

    /// Creates a new instance of the struct with default configuration.
    ///
    /// This function parses the command-line arguments and creates a new instance of the struct
    /// with default configuration. It returns a Result that contains either the new instance or
    /// an error if an error occurs.
    ///
    /// # Examples:
    ///
    /// ```rust
    /// # use crate::MyStruct;
    /// #
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let my_struct = MyStruct::default().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn default() -> Result<Self> {
        // Parse command line arguments
        let args = Args::parse();

        // Create new instance with default configuration
        Self::new(args, true).await
    }

    /// Runs query benchmarks.
    ///
    /// This function:
    ///
    /// 1. Gets the files matching the pattern
    /// 2. Parses the files using a `TomlParser`
    /// 3. Executes benchmark tasks for each query
    /// 4. Returns the results of the query benchmarks as a `QueryBenchResults`
    ///
    /// # Examples
    ///
    /// ```
    /// use qbench::QueryBench;
    /// use std::env::Args;
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut bench = QueryBench::new(Args::parse(),true).await?
    ///     .run_bench().await?;
    ///     let results = bench.run_bench().await.unwrap();
    ///     println!("{:?}", results);
    /// }
    /// ```
    pub async fn run_bench(&mut self) -> Result<Vec<QueryBenchResult>> {
        // Get files that match the pattern
        let files: Vec<PathBuf> = self.get_files_matching_pattern().await?;

        // Initialize parser
        let parser = Arc::new(DefaultParser::new());

        // Create a task for parsing each file
        let mut file_parsing_tasks = FuturesUnordered::new();
        for file in files {
            let parser = parser.clone();
            file_parsing_tasks.push(async move { parser.parse(&file).await });
        }

        // Combine queries from each parsed file
        let mut query_benches = vec![];
        while let Some(query_bench) = file_parsing_tasks.next().await {
            query_benches.append(&mut query_bench?.queries)
        }

        // Create a task for each query benchmark
        let mut query_bench_tasks = FuturesUnordered::new();
        for bench in query_benches {
            let mut self_clone = self.clone();
            query_bench_tasks.push(async move { self_clone.run_query_bench(&bench).await });
        }

        // Collect the results from all query benchmarks
        let mut results = vec![];
        while let Some(result) = query_bench_tasks.next().await {
            results.push(result?);
        }
        // Return the query benchmark results
        Ok(results)
    }

    /// Asynchronously gets a list of files matching a specific glob pattern within a directory.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use qbench::args::Args;
    ///
    /// async fn example() {
    ///   let path = PathBuf::from("./examples");
    ///   let args = Args {
    ///     url: "postgres://user:password@localhost:5432/postgres".to_string(),
    ///     dir: PathBuf::from("./examples"),
    ///     pattern: "*.rs".to_string(),
    ///     max_connections: 10,
    ///     iterations: 10,
    /// };
    ///   let result = get_files_matching_pattern(&args).await;
    ///   assert!(result.is_ok());
    ///   let files = result.unwrap();
    ///   assert!(files.len() > 0);
    /// }
    /// ```
    async fn get_files_matching_pattern(&self) -> Result<Vec<PathBuf>> {
        // Define case insensitive matching options as default
        let glob_options = glob::MatchOptions {
            case_sensitive: false,
            ..Default::default()
        };
        // Clone the arguments and get the directory path
        let args = self.args.clone();
        let dir = args.dir.to_str().unwrap_or("./");

        // Generate the glob pattern from the directory and file pattern
        let pattern = args.filter.clone();
        let glob_path = format!("{}/{}", dir, pattern);

        // Use `glob_with` to fetch all the files that match the pattern
        let files: Vec<PathBuf> = glob_with(glob_path.as_ref(), glob_options)?
            .flatten()
            .filter(|f| f.is_file())
            .collect();
        if files.is_empty() {
            return Err(anyhow!(
                "No files found matching pattern: {} in directory {}",
                pattern,
                dir
            ));
        }
        Ok(files)
    }

    /// Runs query benchmark for given QueryBench, running benchmarks for each revision of query.
    ///
    /// # Arguments
    ///
    /// * `self` - A mutable reference to the current instance of struct implementing QueryRunner trait.
    /// * `bench` - A reference to the QueryBench for which benchmark should be run.
    ///
    /// # Returns
    ///
    /// A Result containing QueryBenchResult on success and corresponding error message on failure.
    ///
    /// # Example
    ///
    /// ```
    /// use futures::executor::block_on;
    ///
    /// let mut runner = MyQueryRunner::new();
    /// let result = block_on(runner.run_query_bench(&bench));
    /// ```
    async fn run_query_bench(&mut self, bench: &QueryBench) -> Result<QueryBenchResult> {
        // Create a new instance of FuturesUnordered to store sub-task of revision benchmarking.
        let mut sub_bench_tasks = FuturesUnordered::new();

        // Iterate through all the revisions in QueryBench and push them into sub_bench_tasks.
        for revision in &bench.revisions {
            // Clone the current instance of struct implementing QueryRunner trait.
            let mut self_clone = self.clone();

            // Create a new async block with move closure, passing the cloned instance of struct.
            sub_bench_tasks.push(async move {
                // Call run_revision_bench on cloned struct instance for current revision of benchmark.
                self_clone.run_revision_bench(revision).await
            });
        }

        // Vector to store QueryBenchResult for each revision.
        let mut results = vec![];

        // Loop through all the completed sub_bench_tasks until no task remains.
        while let Some(result) = sub_bench_tasks.next().await {
            match result {
                Ok(result) => results.push(result),
                Err(e) => {
                    return Err(
                        e.context(format!("Error running benchmark for query {}", bench.name))
                    );
                }
            }
        }

        // Return QueryBenchResult with name of bench and results for each revision.
        Ok(QueryBenchResult {
            name: bench.name.clone(),
            results,
        })
    }

    /// Asynchronously runs benchmark for the provided revision of the query.
    ///
    /// # Arguments
    ///
    /// * `query_revision` - A reference to the query revision for which to run the benchmark.
    ///
    /// # Returns
    ///
    /// Returns a `QueryRevisionResult` enum variant `Success` with the result of the successful benchmark.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use my_crud_library::{QBench, QueryRevision};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// use qbench::bench::QBench;
    /// use qbench::QueryRevision;
    /// let mut qbench = QBench::default().await?;
    ///
    /// let query_revision = QueryRevision {
    ///     name: "test_query".to_string(),
    ///     query: "SELECT * FROM users WHERE email = 'johndoe@example.com'".to_string(),
    ///     pre_script: None,
    ///     post_script: None,
    /// };
    ///
    /// let result = qbench.run_revision_bench(&query_revision).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run_revision_bench(
        &mut self,
        query_revision: &QueryRevision,
    ) -> Result<QueryRevisionResult> {
        // Create a new bench_success_res with the revision name and default values for the rest of the fields
        let mut bench_success_res = QueryRevisionResult {
            revision_name: query_revision.name.clone(),
            ..Default::default()
        };

        // Clone the connection pool
        let pool = self.pool.clone();

        // Create a new Arc<Mutex<_>> wrapping a transaction and clone it
        let tx = Arc::new(Mutex::new(pool.begin().await?));

        // If there is a pre_script, execute it and measure its duration
        if let Some(pre_script) = &query_revision.pre_script {
            bench_success_res.pre_script_duration = QBench::execute_script(pre_script, tx.clone())
                .await
                .map_err(|e| {
                    e.context(format!(
                        "Error executing Pre-Script for revision {}",
                        query_revision.name
                    ))
                })?;
        }

        // Create a vector to store the durations of each iteration
        let mut durations = vec![];

        // Run the benchmark for the specified number of iterations
        for _ in 0..self.args.iterations {
            let start = Instant::now();

            // Lock the transaction and execute the query
            let mut lock = tx.lock().await;
            let _ = query(query_revision.query.as_str())
                .execute(lock.deref_mut())
                .await
                .map_err(|e| {
                    anyhow!(
                        "Error executing query for revision {}: {}",
                        query_revision.name,
                        e
                    )
                })?;

            // Release the lock
            lock.unlock();

            durations.push(start.elapsed());
        }

        // Save the durations to `bench_success_res`
        bench_success_res.durations = durations;

        // Calculate the average duration and save it to `bench_success_res`
        let total = bench_success_res.durations.len() as f64;
        bench_success_res.avg_query_duration = bench_success_res
            .durations
            .iter()
            .sum::<Duration>()
            .div_f64(total);

        // If there is a post_script, execute it and measure its duration
        if let Some(post_script) = &query_revision.post_script {
            bench_success_res.post_script_duration =
                QBench::execute_script(post_script, tx.clone())
                    .await
                    .map_err(|e| {
                        e.context(format!(
                            "Error executing Post-Script for revision {}",
                            query_revision.name
                        ))
                    })?;
        }

        // Rollback the transaction and return the successful result
        Arc::try_unwrap(tx).unwrap().into_inner().rollback().await?;

        Ok(bench_success_res)
    }

    /// Executes a given SQL script in a transaction and returns the execution duration.
    ///
    /// # Arguments
    ///
    /// * `script` - A string slice that represents the SQL script to execute.
    /// * `tx` - An `Arc<Mutex<Transaction<'_, Any>>>` that represents the transaction lock to use
    ///          for executing the script.
    ///
    /// # Example
    ///
    /// ```
    /// use std::sync::{Arc, Mutex};
    /// use sqlx::{Any, query, Transaction};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let db_url = "sqlite::memory:";
    ///     let pool = sqlx::any::AnyPoolOptions::new().connect(db_url).await?;
    ///     let mut tx = pool.begin().await?;
    ///
    ///     let script = "
    ///         CREATE TABLE users (
    ///             id INTEGER PRIMARY KEY,
    ///             name TEXT NOT NULL
    ///         );
    ///
    ///         INSERT INTO users (name) VALUES ('John Doe');
    ///     ";
    ///
    ///     let duration = execute_script(script, Arc::new(Mutex::new(tx))).await?;
    ///
    ///     println!("Execution Duration: {:?}", duration);
    ///
    ///     Ok(())
    /// }
    /// ```
    async fn execute_script(
        script: &str,
        tx: Arc<Mutex<Transaction<'_, Any>>>,
    ) -> Result<Duration> {
        // Record the start time of the function execution.
        let start = Instant::now();

        // Split the given script into individual queries and execute each query in a transaction
        // lock.
        for script_line in extract_multiline_queries(script) {
            let mut lock = tx.lock().await;
            let _ = query(script_line).execute(lock.deref_mut()).await?;

            // Unlock the transaction lock.
            lock.unlock();
        }
        let mut lock = tx.lock().await;
        lock.unlock();
        // Compute the duration of the function execution.
        let duration = start.elapsed();

        // Return the function execution duration.
        Ok(duration)
    }
}
