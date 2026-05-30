use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;

use crate::config::Config;
use crate::generator::llms_full;
use crate::github::client::GitHubClient;
use crate::github::readme;

pub async fn run(path: &str) -> Result<()> {
    let client = GitHubClient::new()?;
    let config = Config::load()?;

    eprintln!("Fetching star lists...");
    let lists = client.fetch_star_lists().await?;
    eprintln!("Fetching all starred (with metadata)...");
    let all_starred = client.fetch_all_starred().await?;

    let total: usize = lists.iter().map(|l| l.repositories.len()).sum();
    eprintln!("Fetching {total} READMEs (sequential)...");

    let http = reqwest::Client::new();
    let token = client.token().to_string();
    let mut readmes: HashMap<String, Option<String>> = HashMap::new();

    let mut fetched = 0usize;
    for list in &lists {
        for repo in &list.repositories {
            let (owner, name) = repo
                .name_with_owner
                .split_once('/')
                .with_context(|| format!("Invalid nameWithOwner {}", repo.name_with_owner))?;
            let result = readme::fetch_readme(&http, &token, owner, name).await?;
            readmes.insert(repo.name_with_owner.clone(), result);
            fetched += 1;
            if fetched.is_multiple_of(50) {
                eprintln!("  {fetched}/{total}");
            }
        }
    }
    eprintln!("  {fetched}/{total} (done)");

    let title = format!("{} stars", client.viewer_login().await?);
    let output = llms_full::generate(&title, &lists, &all_starred, &readmes, &config);
    fs::write(path, output)?;
    println!("Wrote {path}");
    Ok(())
}
