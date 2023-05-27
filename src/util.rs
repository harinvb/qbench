use std::fs::File;
use std::io::Write;
use std::time::Duration;

use anyhow::{anyhow, Result};
use console::Term;
use serde::Serialize;
use tabled::settings::Style;
use tabled::Table;

use crate::{QueryBenchResult, QueryRevisionResult};
use crate::bench::QBench;

/// Extracts multiple queries from a given string, separated by semicolons
///
/// # Examples
///
/// ```
/// let query_str = "SELECT * FROM users WHERE id = 1; SELECT * FROM orders WHERE user_id = 1;";
/// let queries = extract_multiline_queries(query_str);
/// assert_eq!(queries, vec!["SELECT * FROM users WHERE id = 1", "SELECT * FROM orders WHERE user_id = 1"]);
/// ```
///
/// ```
/// let query_str = "SELECT * FROM users WHERE id = 1";
/// let queries = extract_multiline_queries(query_str);
/// assert_eq!(queries, vec!["SELECT * FROM users WHERE id = 1"]);
/// ```
pub fn extract_multiline_queries(query_str: &str) -> Vec<&str> {
    // split the string using `split_inclusive` which will include the separator in the substring.
    // trim each substring to remove leading/trailing white spaces.
    // collect all the substrings as a vector of string slices.
    query_str.split_inclusive(';').map(|s| s.trim()).collect()
}

/// Formats a vector of `QueryRevisionResult` structs into a table using the `Table` library and
/// applies the modern style defined by the `Style` enum, then returns the resulting string.
///
/// # Arguments
///
/// * `rev_result` - A vector of `QueryRevisionResult` structs to be formatted into a table.
///
/// # Example
///
/// ```
/// use crate::QueryRevisionResult;
/// use prettytable::{Style, Table};
///
/// let results = vec![
///     QueryRevisionResult { /* struct fields */ },
///     QueryRevisionResult { /* struct fields */ },
///     QueryRevisionResult { /* struct fields */ }
/// ];
///
/// let formatted_results = format_rev_result(&results);
///
/// println!("{}", formatted_results);
/// ```
pub fn format_rev_result(rev_result: &Vec<QueryRevisionResult>) -> String {
    // Create a new Table object with the given vector of QueryRevisionResults
    let mut table = Table::new(rev_result);

    // Apply the modern style to the table
    table.with(Style::modern()).to_string()
}


/// Converts a Duration value into a human-readable format.
///
/// # Examples
///
/// ```
/// use std::time::Duration;
///
/// let duration = Duration::from_secs(1234567);
/// let pretty_duration = format_duration_pretty(&duration);
/// assert_eq!(pretty_duration, "2w 4d 21h 33m 47s");
/// ```
pub fn format_duration_pretty(duration: &Duration) -> String {
    let mut millis = duration.as_millis();

    // Get the number of milliseconds, seconds, minutes, hours, days, years
    let ms = millis % 1000;
    millis /= 1000;
    let secs = millis % 60;
    millis /= 60;
    let mins = millis % 60;
    millis /= 60;
    let hrs = millis % 24;
    millis /= 24;
    let days = millis;
    millis /= 365;
    let years = millis;

    // Create an empty string to store the result
    let mut res = "".to_string();

    // Append the appropriate unit if the value is greater than zero
    if years > 0 {
        res += &format!("{}y ", years);
    }
    if days > 0 || (years > 0 && hrs > 0) {
        res += &format!("{}d ", days);
    }
    if hrs > 0 || (days > 0 && mins > 0) {
        res += &format!("{}h ", hrs);
    }
    if mins > 0 || (hrs > 0 && secs > 0) {
        res += &format!("{}m ", mins);
    }
    if secs > 0 || (mins > 0 && ms > 0) {
        res += &format!("{}s ", secs);
    }
    if ms > 0 || res.is_empty() {
        res += &format!("{}ms ", ms);
    }

    // Remove any extra whitespace and return the result
    res.trim().to_string()
}


/// Generate file path with extension if not already in the given file path.
///
/// # Arguments
///
/// * `qbench` - A reference to a `QBench` object.
///
/// # Examples
///
/// ```
/// let qbench = QBench::new();
/// qbench.args.out_file = "results";
/// qbench.args.out_format = "json";
/// let out_file_path = out_file(&qbench);
/// assert_eq!(out_file_path, "results.json");
/// ```
fn out_file(qbench: &QBench) -> Result<String> {
    // convert file path to lowercase
    let mut path = qbench.args.out_file.to_lowercase();
    let ext = match qbench.args.export.to_lowercase().as_str() {
        "json" => ".json",
        "toml" => ".toml",
        "none" => "",
        _ => return Err(anyhow!("Invalid export format")),
    };
    // check if file extension is already present
    update_file_extension_if_needed(&mut path, ext);

    Ok(path)
}


/// Updates the file extension of a given file path if it is not already updated.
///
/// # Arguments
///
/// * `path` - A mutable reference to a string representing the file path.
/// * `ext` - An immutable reference to a string representing the file extension to be added if needed.
///
/// # Example
///
/// ```
/// let mut path = String::from("example.txt");
/// update_file_extension_if_needed(&mut path, ".txt");
/// assert_eq!(path, String::from("example.txt"));
///
/// let mut path = String::from("example");
/// update_file_extension_if_needed(&mut path, ".txt");
/// assert_eq!(path, String::from("example.txt"));
/// ```
fn update_file_extension_if_needed(path: &mut String, ext: &str) {
    // Check if the file path ends with the given extension
    if !path.ends_with(ext) {
        // If not, add the extension to the file path
        path.push_str(ext);
    }
}


/// Struct representing the exported query benchmark results.
#[derive(Serialize)]
struct ExportedQBenchResults<'a> {
    exported: &'a Vec<QueryBenchResult>,
}

/// Exports the query benchmark results to a TOML file.
///
/// # Arguments
///
/// * `term` - Terminal interface for displaying progress and status messages.
/// * `qbench` - Query benchmark configuration.
/// * `res` - Query benchmark results to be exported.
///
/// # Example
///
/// ```
/// let term = Term::stdout();
/// let qbench = QBench::default();
/// let results = vec![QueryBenchResult::new("SELECT * FROM users", 1.0, 100)];
///
/// export_toml(&term, &qbench, &results).expect("Failed to export results.");
/// ```
pub fn export_toml(term: &Term, qbench: &QBench, res: &Vec<QueryBenchResult>) -> Result<()> {
    term.write_line("Exporting results to TOML...")?;
    let mut file = File::create(out_file(qbench)?)?;

    let results = ExportedQBenchResults {
        exported: res
    };

    writeln!(file, "{}", toml::to_string_pretty(&results)?)?;
    term.clear_last_lines(1)?;
    term.write_line("Results exported to TOML.")?;
    Ok(())
}

/// Exports the query benchmark results to a JSON file.
///
/// # Arguments
///
/// * `term` - Terminal interface for displaying progress and status messages.
/// * `qbench` - Query benchmark configuration.
/// * `bench_res` - Query benchmark results to be exported.
///
/// # Example
///
/// ```
/// let term = Term::stdout();
/// let qbench = QBench::default();
/// let results = vec![QueryBenchResult::new("SELECT * FROM users", 1.0, 100)];
///
/// export_json(&term, &qbench, &results).expect("Failed to export results.");
/// ```
pub fn export_json(term: &Term, qbench: &QBench, bench_res: &Vec<QueryBenchResult>) -> Result<()> {
    term.write_line("Exporting results to JSON...")?;

    let exported = ExportedQBenchResults {
        exported: bench_res
    };

    serde_json::to_writer_pretty(File::create(out_file(qbench)?)?, &exported)?;
    term.clear_last_lines(1)?;
    term.write_line("Results exported to JSON.")?;
    Ok(())
}
