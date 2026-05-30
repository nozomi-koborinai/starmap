use crate::generator;
use crate::github::client::GitHubClient;

pub async fn run() -> anyhow::Result<()> {
    let client = GitHubClient::new()?;
    let star_lists = client.fetch_star_lists().await?;
    let all_starred = client.fetch_all_starred().await?;
    let markdown =
        generator::markdown::generate(&star_lists, &all_starred, &crate::config::Config::default());
    print!("{}", markdown);
    Ok(())
}
