use anyhow::Result;
use clap::Parser;
use console::{style, Term};
use tabled::{settings::Style, Table};

use qbench::args::Args;
use qbench::bench::QBench;
use qbench::util;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let term = Term::stdout();
    term.write_line("Running benchmarks...")?;
    let mut qbench = QBench::new(args, true).await?;
    let bench_res = qbench.run_bench().await;
    term.clear_last_lines(1)?;

    match (bench_res, qbench.args.export.to_lowercase().as_str()) {
        (Ok(bench_res), "json") => {
            util::export_json(&term, &qbench, &bench_res)?;
        }
        (Ok(bench_res), "toml") => {
            util::export_toml(&term, &qbench, &bench_res)?;
        }
        (Ok(bench_res), _) => {
            let mut table = Table::new(bench_res);
            table.with(Style::modern());
            term.write_line(&table.to_string())?;
        }
        (Err(e), _) => {
            term.write_line(
                style(format!("{:?}", e).as_str())
                    .red()
                    .to_string()
                    .as_str(),
            )?;
        }
    }

    Ok(())
}
