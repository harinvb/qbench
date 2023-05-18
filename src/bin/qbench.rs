use anyhow::Result;
use clap::Parser;

use qbench::args::Args;
use qbench::bench::run_bench;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let results = run_bench(&args).await?;
    //TODO: print results to terminal using some TUI library
    println!("{:#?}", results);
    Ok(())
}
