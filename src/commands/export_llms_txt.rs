use anyhow::Result;
use std::fs;

use crate::config::Config;
use crate::generator::llms_txt;
use crate::github::client::GitHubClient;

pub async fn run(path: &str) -> Result<()> {
    let client = GitHubClient::new()?;
    let lists = client.fetch_star_lists().await?;
    let config = Config::load()?;
    let title = format!("{} stars", client.viewer_login().await?);
    let output = llms_txt::generate(&title, &lists, &config);
    fs::write(path, output)?;
    println!("Wrote {path}");
    Ok(())
}
