use anyhow::{bail, Context};
use octocrab::Octocrab;

use crate::generator;
use crate::github::client::GitHubClient;

pub async fn run(repo: &str) -> anyhow::Result<()> {
    let (owner, name) = repo
        .split_once('/')
        .context("Invalid repo format. Expected owner/name")?;

    // Generate markdown
    let config = crate::config::Config::load()?;
    let client = GitHubClient::new()?;
    let star_lists = client.fetch_star_lists().await?;
    let all_starred = client.fetch_all_starred().await?;
    let markdown = generator::markdown::generate(&star_lists, &all_starred, &config);

    // Push to GitHub
    let token = resolve_token()?;
    let octo = Octocrab::builder()
        .personal_token(token)
        .build()
        .context("Failed to build Octocrab client")?;

    let repos = octo.repos(owner, name);
    let path = "README.md";
    let message = "docs: update awesome list via starmap";

    // Check if file already exists to get SHA
    match repos.get_content().path(path).send().await {
        Ok(contents) => {
            if let Some(existing) = contents.items.first() {
                let sha = existing.sha.as_str();
                repos
                    .update_file(path, message, &markdown, sha)
                    .send()
                    .await
                    .context("Failed to update README.md")?;
            } else {
                bail!("Unexpected empty content response");
            }
        }
        Err(_) => {
            // File doesn't exist yet, create it
            repos
                .create_file(path, message, &markdown)
                .send()
                .await
                .context("Failed to create README.md")?;
        }
    }

    eprintln!("Pushed to {repo}");
    Ok(())
}

fn resolve_token() -> anyhow::Result<String> {
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        if !token.is_empty() {
            return Ok(token);
        }
    }

    let output = std::process::Command::new("gh")
        .args(["auth", "token"])
        .output()
        .context("Failed to run `gh auth token`")?;

    if output.status.success() {
        let token = String::from_utf8(output.stdout)?.trim().to_string();
        if !token.is_empty() {
            return Ok(token);
        }
    }

    bail!("No GitHub token found. Set GITHUB_TOKEN or run `gh auth login`.")
}
