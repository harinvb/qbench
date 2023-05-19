use anyhow::Result;
use clap::Parser;
use console::{style, Term};
use tabled::settings::Style;
use tabled::Table;

use qbench::args::Args;
use qbench::bench::QBench;

#[tokio::main]
async fn main() -> Result<()> {
    let term = Term::stdout();
    term.write_line("Running bench marks...")?;
    let bench_res = QBench::new(Args::parse(), true).await?.run_bench().await;
    term.clear_last_lines(1)?;
    match bench_res {
        Ok(bench_res) => {
            let mut table = Table::new(bench_res);
            table.with(Style::modern());
            term.write_line(&table.to_string())?;
        }
        Err(e) => {
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
