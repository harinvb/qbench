use std::path::PathBuf;

use clap::Parser;

/// The following code defines a struct called Args which is used for parsing command line arguments.
///
/// It derives two traits: Debug and Parser. Debug prints a debug representation of the struct,
/// and Parser signals that the struct should be used for parsing arguments.
#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Args {
    /// A String representation of the database connection URL.
    #[arg(
        short = 'u',
        long = "url",
        default_value = "postgres://user:password@localhost:5432/postgres"
    )]
    pub url: String,

    /// A Path representation of the directory from where the benchmark config will be loaded.
    #[arg(short = 'd', long = "bench-dir", default_value = "./")]
    pub dir: PathBuf,

    /// A String pattern that represents the file extension(s) to benchmark.
    /// Currently only supports toml format.
    #[arg(short = 'p', long = "pattern", default_value = "*.toml")]
    pub pattern: String,

    /// An unsigned 32-bit integer that represents the maximum number of connections.
    #[arg(short = 'c', long = "max-connections", default_value = "10")]
    pub max_connections: u32,

    /// An unsigned integer that represents the number of iterations to perform on
    /// each revision.
    #[arg(short = 'i', long = "iterations", default_value = "1")]
    pub iterations: usize,
}
