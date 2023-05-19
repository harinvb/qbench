use std::path::PathBuf;

use clap::Parser;
/// Doc comments can be placed here to describe the purpose of this module, crate, or item.
/// The following code defines a struct called Args which is used for parsing command line arguments.
///
/// It derives two traits: Debug and Parser. Debug prints a debug representation of the struct,
/// and Parser signals that the struct should be used for parsing arguments.
#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Args {
    /// A String representation of the database connection URL.
    /// This field can be set via the `-u` or `--url` command-line option
    /// and defaults to `postgres://user:password@localhost:5432/postgres`.
    #[arg(
        short = 'u',
        long = "url",
        default_value = "postgres://user:password@localhost:5432/postgres"
    )]
    pub url: String,

    /// A PathBuf representation of the directory where the benchmark will be saved.
    /// This field can be set via the `-d` or `--bench-dir` command-line option
    /// and defaults to `"./"`.
    #[arg(short = 'd', long = "bench-dir", default_value = "./")]
    pub dir: PathBuf,

    /// A String pattern that represents the file extension(s) to benchmark.
    /// This field can be set via the `-p` or `--pattern` command-line option
    /// and defaults to `"*.toml"`.
    #[arg(short = 'p', long = "pattern", default_value = "*.toml")]
    pub pattern: String,

    /// An unsigned 32-bit integer that represents the maximum number of connections.
    /// This field can be set via the `-c` or `--max-connections` command-line option
    /// and defaults to `10`.
    #[arg(short = 'c', long = "max-connections", default_value = "10")]
    pub max_connections: u32,

    /// An unsigned integer that represents the number of iterations to perform.
    /// This field can be set via the `-i` or `--iterations` command-line option
    /// and defaults to `1`.
    #[arg(short = 'i', long = "iterations", default_value = "1")]
    pub iterations: usize,
}
