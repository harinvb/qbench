use std::path::PathBuf;

use clap::Parser;

/// The following code defines a struct called Args which is used for parsing command line arguments.
///
/// It derives two traits: Debug and Parser. Debug prints a debug representation of the struct,
/// and Parser signals that the struct should be used for parsing arguments.
#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Args {
    ///The database connection URL.
    #[arg(
        short = 'u',
        long = "url",
        default_value = "postgres://user:password@localhost:5432/postgres"
    )]
    pub url: String,

    /// Directory from where the benchmark config will be loaded.
    #[arg(short = 'd', long = "bench-dir", default_value = "./")]
    pub dir: PathBuf,

    /// The config file filter.
    /// Currently only supports parsing toml,json format.
    #[arg(short = 'f', long = "filter", default_value = "*.toml")]
    pub filter: String,

    /// The maximum number of connections.
    #[arg(short = 'c', long = "max-connections", default_value = "100")]
    pub max_connections: u32,

    /// Number of iterations to perform on
    /// each revision.
    #[arg(short = 'i', long = "iterations", default_value = "1")]
    pub iterations: usize,

    /// Specifies how to export (e.g. 'json', 'toml', 'none').
    #[arg(short = 'e', long = "export", default_value = "none")]
    pub export: String,

    /// The output file.
    #[arg(short = 'o', long = "out-file", default_value = "out")]
    pub out_file: String,

    /// The maximum time to wait for a database connection to be available.
    #[arg(long = "connection-acquire-timeout", default_value = "180")]
    pub connection_acquire_timeout: u64,

    /// The maximum time to keep an idle database connection before closing it.
    #[arg(long = "connection-idle-timeout", default_value = "180")]
    pub connection_idle_timeout: u64,
}
