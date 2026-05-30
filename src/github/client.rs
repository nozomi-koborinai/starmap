use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde_json::{json, Value};
use std::process::Command;

use super::types::*;

const GRAPHQL_ENDPOINT: &str = "https://api.github.com/graphql";

pub struct GitHubClient {
    http: Client,
    token: String,
}

impl GitHubClient {
    pub fn new() -> Result<Self> {
        let token = resolve_token()?;
        let http = Client::new();
        Ok(Self { http, token })
    }

    /// Fetch all Star Lists and their repositories
    pub async fn fetch_star_lists(&self) -> Result<Vec<StarList>> {
        let mut lists = Vec::new();
        let mut cursor: Option<String> = None;

        loop {
            let query = build_lists_query(&cursor);
            let data: ListsQueryData = self.execute_query(&query).await?;
            let conn = data.viewer.lists;

            for raw_list in conn.nodes {
                let mut star_list = StarList::from(raw_list.clone());

                // Fetch remaining pages if items are paginated
                if raw_list.items.page_info.has_next_page {
                    let remaining = self
                        .fetch_remaining_list_items(
                            &raw_list.id,
                            raw_list.items.page_info.end_cursor,
                        )
                        .await?;
                    star_list.repositories.extend(remaining);
                }

                lists.push(star_list);
            }

            if !conn.page_info.has_next_page {
                break;
            }
            cursor = conn.page_info.end_cursor;
        }

        Ok(lists)
    }

    /// Fetch all starred repositories
    pub async fn fetch_all_starred(&self) -> Result<Vec<Repository>> {
        let mut repos = Vec::new();
        let mut cursor: Option<String> = None;

        loop {
            let query = build_starred_query(&cursor);
            let data: StarredQueryData = self.execute_query(&query).await?;
            let conn = data.viewer.starred_repositories;

            repos.extend(conn.nodes.into_iter().map(Repository::from));

            if !conn.page_info.has_next_page {
                break;
            }
            cursor = conn.page_info.end_cursor;
        }

        Ok(repos)
    }

    /// Fetch remaining pages of items within a list
    async fn fetch_remaining_list_items(
        &self,
        list_id: &str,
        start_cursor: Option<String>,
    ) -> Result<Vec<Repository>> {
        let mut repos = Vec::new();
        let mut cursor = start_cursor;

        loop {
            let query = build_list_items_query(list_id, &cursor);
            let data: ListItemsQueryData = self.execute_query(&query).await?;
            let conn = data.node.items;

            repos.extend(conn.nodes.into_iter().map(Repository::from));

            if !conn.page_info.has_next_page {
                break;
            }
            cursor = conn.page_info.end_cursor;
        }

        Ok(repos)
    }

    /// Execute a GraphQL query and deserialize the response
    async fn execute_query<T: serde::de::DeserializeOwned>(&self, query: &str) -> Result<T> {
        let body = json!({ "query": query });

        let resp = self
            .http
            .post(GRAPHQL_ENDPOINT)
            .bearer_auth(&self.token)
            .header("User-Agent", "starmap-cli")
            .json(&body)
            .send()
            .await
            .context("Failed to send GraphQL request")?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            bail!("GitHub API returned {status}: {text}");
        }

        let json: Value = resp.json().await.context("Failed to parse response JSON")?;

        if let Some(errors) = json.get("errors") {
            bail!("GraphQL errors: {errors}");
        }

        let data = json
            .get("data")
            .context("No 'data' field in GraphQL response")?
            .clone();

        serde_json::from_value(data).context("Failed to deserialize GraphQL data")
    }
}

// ---------------------------------------------------------------------------
// Token resolution
// ---------------------------------------------------------------------------

fn resolve_token() -> Result<String> {
    // 1. Prefer environment variable
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        if !token.is_empty() {
            return Ok(token);
        }
    }

    // 2. Fall back to `gh auth token`
    let output = Command::new("gh")
        .args(["auth", "token"])
        .output()
        .context("Failed to run `gh auth token`. Is gh CLI installed?")?;

    if output.status.success() {
        let token = String::from_utf8(output.stdout)
            .context("Invalid UTF-8 from gh auth token")?
            .trim()
            .to_string();
        if !token.is_empty() {
            return Ok(token);
        }
    }

    bail!("No GitHub token found. Set GITHUB_TOKEN or run `gh auth login`.")
}

// ---------------------------------------------------------------------------
// GraphQL query builders
// ---------------------------------------------------------------------------

fn build_lists_query(cursor: &Option<String>) -> String {
    let after = cursor_arg(cursor);
    format!(
        r#"{{
  viewer {{
    lists(first: 100{after}) {{
      totalCount
      nodes {{
        id
        name
        description
        isPrivate
        items(first: 100) {{
          totalCount
          nodes {{
            ... on Repository {{
              nameWithOwner
              description
              url
            }}
          }}
          pageInfo {{ hasNextPage endCursor }}
        }}
      }}
      pageInfo {{ hasNextPage endCursor }}
    }}
  }}
}}"#
    )
}

fn build_starred_query(cursor: &Option<String>) -> String {
    let after = cursor_arg(cursor);
    format!(
        r#"{{
  viewer {{
    starredRepositories(first: 100{after}) {{
      nodes {{
        nameWithOwner
        description
        url
        stargazerCount
        primaryLanguage {{ name }}
        repositoryTopics(first: 10) {{
          nodes {{ topic {{ name }} }}
        }}
      }}
      pageInfo {{ hasNextPage endCursor }}
    }}
  }}
}}"#
    )
}

fn build_list_items_query(list_id: &str, cursor: &Option<String>) -> String {
    let after = cursor_arg(cursor);
    format!(
        r#"{{
  node(id: "{list_id}") {{
    ... on UserList {{
      items(first: 100{after}) {{
        totalCount
        nodes {{
          ... on Repository {{
            nameWithOwner
            description
            url
          }}
        }}
        pageInfo {{ hasNextPage endCursor }}
      }}
    }}
  }}
}}"#
    )
}

fn cursor_arg(cursor: &Option<String>) -> String {
    match cursor {
        Some(c) => format!(", after: \"{c}\""),
        None => String::new(),
    }
}
