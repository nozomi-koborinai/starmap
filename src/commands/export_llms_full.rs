use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::time::Duration;

use crate::config::Config;
use crate::generator::llms_full;
use crate::github::client::GitHubClient;
use crate::github::readme;

/// Throttle between README fetches to stay under GitHub's secondary rate
/// limit (undocumented but ~80 req/min for content endpoints). 250ms gives
/// ~240 req/min and burns ~3 minutes for 700 repos.
const FETCH_INTERVAL: Duration = Duration::from_millis(250);

pub async fn run(path: &str) -> Result<()> {
    let client = GitHubClient::new()?;
    let config = Config::load()?;

    eprintln!("Fetching star lists...");
    let lists = client.fetch_star_lists().await?;
    eprintln!("Fetching all starred (with metadata)...");
    let all_starred = client.fetch_all_starred().await?;

    let total: usize = lists.iter().map(|l| l.repositories.len()).sum();
    eprintln!("Fetching {total} READMEs (throttled at {FETCH_INTERVAL:?})...");

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
            tokio::time::sleep(FETCH_INTERVAL).await;
        }
    }
    eprintln!("  {fetched}/{total} (done)");

    let title = format!("{} stars", client.viewer_login().await?);
    let output = llms_full::generate(&title, &lists, &all_starred, &readmes, &config);
    fs::write(path, output)?;
    println!("Wrote {path}");
    Ok(())
}
