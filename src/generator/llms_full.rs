use std::collections::HashMap;

use crate::config::Config;
use crate::github::readme::truncate_utf8;
use crate::github::types::{Repository, StarList};

/// Render the llms-full archive. `readmes` maps `nameWithOwner` -> README content or None.
pub fn generate(
    title: &str,
    lists: &[StarList],
    all_starred: &[Repository],
    readmes: &HashMap<String, Option<String>>,
    config: &Config,
) -> String {
    let mut out = String::new();
    out.push_str(&format!("# {title} — Full archive\n\n"));
    out.push_str(&format!(
        "> Description, topics, and README of each starred repository.\n> Capped at {} KB per README.\n\n",
        config.llms_full.max_readme_size_kb
    ));

    let metadata: HashMap<&str, &Repository> = all_starred
        .iter()
        .map(|r| (r.name_with_owner.as_str(), r))
        .collect();

    let ordered = apply_order(lists, &config.order);

    for list in ordered {
        out.push_str(&format!("## {}\n\n", list.name));
        let mut repos: Vec<&Repository> = list.repositories.iter().collect();
        repos.sort_by(|a, b| {
            a.name_with_owner
                .to_lowercase()
                .cmp(&b.name_with_owner.to_lowercase())
        });
        for repo in repos {
            // Prefer metadata from all_starred (richer); fall back to list entry.
            let r = metadata
                .get(repo.name_with_owner.as_str())
                .copied()
                .unwrap_or(repo);
            render_repo(&mut out, r, readmes, config.llms_full.max_readme_size_kb);
        }
    }
    out
}

fn render_repo(
    out: &mut String,
    r: &Repository,
    readmes: &HashMap<String, Option<String>>,
    max_kb: usize,
) {
    out.push_str(&format!("### [{}]({})\n", r.name_with_owner, r.url));
    if let Some(desc) = &r.description {
        if !desc.is_empty() {
            out.push_str(&format!("- **Description:** {desc}\n"));
        }
    }
    if !r.topics.is_empty() {
        out.push_str(&format!("- **Topics:** {}\n", r.topics.join(", ")));
    }
    if let Some(lang) = &r.language {
        out.push_str(&format!("- **Language:** {lang}\n"));
    }
    if let Some(stars) = r.stargazer_count {
        out.push_str(&format!("- **Stars:** {stars}\n"));
    }
    out.push('\n');

    let max_bytes = max_kb.saturating_mul(1024);
    match readmes.get(&r.name_with_owner) {
        Some(Some(content)) => {
            let truncated = truncate_utf8(content, max_bytes);
            out.push_str("<details>\n<summary>README</summary>\n\n");
            out.push_str(truncated);
            if content.len() > truncated.len() {
                out.push_str(&format!(
                    "\n\n*(truncated at {max_kb} KB; see source repo for full README)*\n"
                ));
            }
            out.push_str("\n</details>\n\n");
        }
        Some(None) | None => {
            out.push_str("*(README unavailable)*\n\n");
        }
    }
    out.push_str("---\n\n");
}

fn apply_order<'a>(lists: &'a [StarList], order: &[String]) -> Vec<&'a StarList> {
    if order.is_empty() {
        return lists.iter().collect();
    }
    let mut remaining: HashMap<&str, &StarList> =
        lists.iter().map(|l| (l.name.as_str(), l)).collect();
    let mut ordered = Vec::with_capacity(lists.len());
    for name in order {
        if let Some(l) = remaining.remove(name.as_str()) {
            ordered.push(l);
        }
    }
    for l in lists {
        if remaining.contains_key(l.name.as_str()) {
            ordered.push(l);
        }
    }
    ordered
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo(nwo: &str) -> Repository {
        Repository {
            name_with_owner: nwo.into(),
            description: Some("d".into()),
            url: format!("https://github.com/{nwo}"),
            stargazer_count: Some(42),
            language: Some("Rust".into()),
            topics: vec!["a".into(), "b".into()],
        }
    }

    #[test]
    fn renders_header_and_metadata() {
        let lists = vec![StarList {
            name: "🤖 AI".into(),
            description: None,
            repositories: vec![repo("x/y")],
        }];
        let all_starred = vec![repo("x/y")];
        let mut readmes = HashMap::new();
        readmes.insert("x/y".into(), Some("README body".into()));
        let out = generate("test", &lists, &all_starred, &readmes, &Config::default());
        assert!(out.contains("# test — Full archive"));
        assert!(out.contains("## 🤖 AI"));
        assert!(out.contains("### [x/y]"));
        assert!(out.contains("**Description:** d"));
        assert!(out.contains("**Topics:** a, b"));
        assert!(out.contains("**Language:** Rust"));
        assert!(out.contains("**Stars:** 42"));
        assert!(out.contains("<details>\n<summary>README</summary>"));
        assert!(out.contains("README body"));
    }

    #[test]
    fn truncates_long_readme() {
        let lists = vec![StarList {
            name: "X".into(),
            description: None,
            repositories: vec![repo("x/y")],
        }];
        let all_starred = vec![repo("x/y")];
        let mut readmes = HashMap::new();
        readmes.insert("x/y".into(), Some("a".repeat(20_000)));
        let cfg = Config {
            llms_full: crate::config::LlmsFullConfig {
                max_readme_size_kb: 5,
            },
            ..Default::default()
        };
        let out = generate("t", &lists, &all_starred, &readmes, &cfg);
        assert!(out.contains("truncated at 5 KB"));
    }

    #[test]
    fn missing_readme_shown_as_unavailable() {
        let lists = vec![StarList {
            name: "X".into(),
            description: None,
            repositories: vec![repo("x/y")],
        }];
        let readmes = HashMap::new();
        let out = generate("t", &lists, &[repo("x/y")], &readmes, &Config::default());
        assert!(out.contains("*(README unavailable)*"));
    }
}
