use anyhow::Result;
use clap::Parser;

use qbench::args::Args;
use qbench::bench::QBench;
use qbench::QueryBenchResults;

#[tokio::main]
async fn main() -> Result<()> {
    let results: QueryBenchResults = QBench::new(Args::parse(),
                                                 true)
        .await?
        .run_bench()
        .await?;
    //TODO: print results to terminal using some TUI library
    println!("{:#?}", results);
    Ok(())
}
