use std::collections::HashSet;

use crate::github::types::{Repository, StarList};

/// Generate an Awesome List formatted Markdown from Star Lists and all starred repos.
///
/// `lists` — Star Lists (each containing its repositories)
/// `all_starred` — All starred repositories (regardless of list membership)
///
/// Stars not belonging to any list are grouped under an "Uncategorized" section.
pub fn generate(lists: &[StarList], all_starred: &[Repository]) -> String {
    let mut out = String::new();

    // Header
    out.push_str("# Awesome Stars\n\n");
    out.push_str("> A curated list of my GitHub stars, organized by lists.\n\n");

    // Compute uncategorized stars
    let listed_repos: HashSet<&str> = lists
        .iter()
        .flat_map(|l| l.repositories.iter())
        .map(|r| r.name_with_owner.as_str())
        .collect();

    let mut uncategorized: Vec<&Repository> = all_starred
        .iter()
        .filter(|r| !listed_repos.contains(r.name_with_owner.as_str()))
        .collect();
    uncategorized.sort_by(|a, b| {
        a.name_with_owner
            .to_lowercase()
            .cmp(&b.name_with_owner.to_lowercase())
    });

    // Contents (TOC)
    out.push_str("## Contents\n\n");
    for list in lists {
        let anchor = to_anchor(&list.name);
        out.push_str(&format!("- [{}](#{})\n", list.name, anchor));
    }
    if !uncategorized.is_empty() {
        out.push_str("- [Uncategorized](#uncategorized)\n");
    }
    out.push('\n');

    // List sections
    for list in lists {
        out.push_str(&format!("## {}\n\n", list.name));
        if let Some(desc) = &list.description {
            if !desc.is_empty() {
                out.push_str(&format!("> {desc}\n\n"));
            }
        }
        let mut repos: Vec<&Repository> = list.repositories.iter().collect();
        repos.sort_by(|a, b| {
            a.name_with_owner
                .to_lowercase()
                .cmp(&b.name_with_owner.to_lowercase())
        });
        for repo in &repos {
            write_repo_line(&mut out, repo);
        }
        out.push('\n');
    }

    // Uncategorized section
    if !uncategorized.is_empty() {
        out.push_str("## Uncategorized\n\n");
        for repo in &uncategorized {
            write_repo_line(&mut out, repo);
        }
        out.push('\n');
    }

    out
}

fn write_repo_line(out: &mut String, repo: &Repository) {
    let desc = repo
        .description
        .as_deref()
        .unwrap_or("No description provided");
    out.push_str(&format!(
        "- [{}]({}) - {}\n",
        repo.name_with_owner, repo.url, desc
    ));
}

/// Convert a section name to a GitHub-compatible anchor
fn to_anchor(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == ' ' {
                c
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}

/// Return true when the list name designates a Focus List.
/// A Focus List name starts with `Focus: ` (capital F, colon, space),
/// optionally preceded by an emoji prefix (single non-ASCII char + ASCII space).
#[allow(dead_code)]
fn is_focus_list(name: &str) -> bool {
    strip_emoji_prefix(name).starts_with("Focus: ")
}

/// If `name` starts with a non-ASCII character followed by an ASCII space,
/// return the remainder after that space. Otherwise return `name` unchanged.
#[allow(dead_code)]
fn strip_emoji_prefix(name: &str) -> &str {
    let mut chars = name.char_indices();
    let Some((_, first)) = chars.next() else {
        return name;
    };
    if first.is_ascii() {
        return name;
    }
    let Some((idx, ' ')) = chars.next() else {
        return name;
    };
    &name[idx + 1..]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::types::{Repository, StarList};

    fn make_repo(name: &str, desc: Option<&str>) -> Repository {
        Repository {
            name_with_owner: name.to_string(),
            description: desc.map(|s| s.to_string()),
            url: format!("https://github.com/{name}"),
        }
    }

    #[test]
    fn test_generate_basic() {
        let lists = vec![StarList {
            name: "Rust Tools".to_string(),
            description: Some("Useful Rust tools".to_string()),
            repositories: vec![
                make_repo("BurntSushi/ripgrep", Some("Fast search")),
                make_repo("sharkdp/bat", Some("A cat clone")),
            ],
        }];

        let all_starred = vec![
            make_repo("BurntSushi/ripgrep", Some("Fast search")),
            make_repo("sharkdp/bat", Some("A cat clone")),
            make_repo("astral-sh/uv", Some("Python package manager")),
        ];

        let md = generate(&lists, &all_starred);

        assert!(md.contains("# Awesome Stars"));
        assert!(md.contains("## Contents"));
        assert!(md.contains("- [Rust Tools](#rust-tools)"));
        assert!(md.contains("- [Uncategorized](#uncategorized)"));
        assert!(md.contains("## Rust Tools"));
        assert!(md.contains("> Useful Rust tools"));
        assert!(md
            .contains("[BurntSushi/ripgrep](https://github.com/BurntSushi/ripgrep) - Fast search"));
        assert!(md.contains("## Uncategorized"));
        assert!(
            md.contains("[astral-sh/uv](https://github.com/astral-sh/uv) - Python package manager")
        );
    }

    #[test]
    fn test_generate_no_uncategorized() {
        let lists = vec![StarList {
            name: "All".to_string(),
            description: None,
            repositories: vec![make_repo("owner/repo", Some("desc"))],
        }];

        let all_starred = vec![make_repo("owner/repo", Some("desc"))];

        let md = generate(&lists, &all_starred);

        assert!(!md.contains("Uncategorized"));
    }

    #[test]
    fn test_alphabetical_sort() {
        let lists = vec![StarList {
            name: "Test".to_string(),
            description: None,
            repositories: vec![
                make_repo("zzz/repo", Some("last")),
                make_repo("aaa/repo", Some("first")),
            ],
        }];

        let md = generate(&lists, &[]);
        let pos_a = md.find("aaa/repo").unwrap();
        let pos_z = md.find("zzz/repo").unwrap();
        assert!(pos_a < pos_z);
    }

    #[test]
    fn test_is_focus_list_plain_prefix() {
        assert!(is_focus_list("Focus: In Production"));
    }

    #[test]
    fn test_is_focus_list_with_emoji_prefix() {
        assert!(is_focus_list("🔥 Focus: In Production"));
    }

    #[test]
    fn test_is_focus_list_lowercase_is_topic() {
        assert!(!is_focus_list("focus: In Production"));
    }

    #[test]
    fn test_is_focus_list_no_colon_is_topic() {
        assert!(!is_focus_list("Focus In Production"));
    }

    #[test]
    fn test_is_focus_list_plain_topic() {
        assert!(!is_focus_list("🤖 AI Frameworks"));
    }

    #[test]
    fn test_is_focus_list_bare_prefix_is_focus() {
        // Bare "Focus: " (no display name) is still detected as focus.
        // The caller (legend emission) is responsible for filtering empty-display lists.
        assert!(is_focus_list("Focus: "));
    }
}
