use std::path::Path;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use tokio::fs::read_to_string;

use crate::{QueryBenches, QueryBenchParser};

pub struct DefaultParser {}

impl DefaultParser {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl QueryBenchParser for DefaultParser {
    async fn parse(&self, path: &Path) -> Result<QueryBenches> {
        let file_content = read_to_string(path).await?;
        match path.extension() {
            Some(ext) => match ext.to_str() {
                Some("json") => {
                    let qb: QueryBenches = serde_json::from_str(file_content.as_str())?;
                    Ok(qb)
                }
                Some("toml") => {
                    let qb: QueryBenches = toml::from_str(file_content.as_str())?;
                    Ok(qb)
                }
                _ => return Err(anyhow!("Unsupported file extension: {}", path.display())),
            },
            _ => {
                return Err(anyhow!(
                    "File has no extension, cannot determine parser: {}",
                    path.display()
                ));
            }
        }
    }
}