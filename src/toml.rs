use std::path::Path;

use anyhow::{anyhow, Result};
use async_std::fs::read_to_string;

use crate::{QueryBenches, QueryBenchParser};

/// A parser for TOML files which implements the QueryBenchParser trait.
#[derive(Debug, Default)]
pub struct TomlParser {}

impl TomlParser {
    /// Creates a new instance of the TomlParser.
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl QueryBenchParser for TomlParser {
    /// Parses a TOML file and returns a `QueryBenches` result.
    async fn parse(&self, path: &Path) -> Result<QueryBenches> {
        let qb: QueryBenches = toml::from_str(read_to_string(path).await?.as_str()) // parse TOML string to QueryBenches object
            .map_err(|_| anyhow!("Failed to parse toml file: {}", path.display()))?; // handle parsing error
        Ok(qb)
    }
}
