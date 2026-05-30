use std::collections::{HashMap, HashSet};

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

/// Build the display name for a Focus List:
///   1. Slice off the emoji prefix (if present)
///   2. Strip the leading `Focus: ` from the remainder
///   3. Re-attach the emoji prefix
///
/// Callers should treat a trimmed-empty result as an invalid Focus List.
fn focus_display_name(name: &str) -> String {
    let rest = strip_emoji_prefix(name);
    let stripped = rest.strip_prefix("Focus: ").unwrap_or(rest);
    let emoji_part = &name[..name.len() - rest.len()];
    format!("{emoji_part}{stripped}")
}

/// Build a map from a repo's `name_with_owner` to the Focus display tags it carries.
/// Tags appear in the order the Focus Lists are given in the input slice.
/// Focus Lists with an empty display name (after `focus_display_name`) are silently
/// skipped — the caller is responsible for surfacing a warning.
#[allow(dead_code)]
fn build_focus_index(focus_lists: &[&StarList]) -> HashMap<String, Vec<String>> {
    let mut index: HashMap<String, Vec<String>> = HashMap::new();
    for list in focus_lists {
        let display = focus_display_name(&list.name);
        if display.trim().is_empty() {
            continue;
        }
        for repo in &list.repositories {
            index
                .entry(repo.name_with_owner.clone())
                .or_default()
                .push(display.clone());
        }
    }
    index
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

    #[test]
    fn test_focus_display_name_plain() {
        assert_eq!(focus_display_name("Focus: In Production"), "In Production");
    }

    #[test]
    fn test_focus_display_name_with_emoji() {
        assert_eq!(
            focus_display_name("🔥 Focus: In Production"),
            "🔥 In Production"
        );
    }

    #[test]
    fn test_focus_display_name_bare_with_emoji() {
        // After stripping, only "emoji + space" remains; trim() would yield empty.
        assert_eq!(focus_display_name("🔥 Focus: "), "🔥 ");
    }

    #[test]
    fn test_focus_display_name_bare_plain() {
        assert_eq!(focus_display_name("Focus: "), "");
    }

    #[test]
    fn test_build_focus_index_single_list() {
        let focus = StarList {
            name: "🔥 Focus: In Production".to_string(),
            description: None,
            repositories: vec![make_repo("a/b", None), make_repo("c/d", None)],
        };
        let index = build_focus_index(&[&focus]);
        assert_eq!(
            index.get("a/b"),
            Some(&vec!["🔥 In Production".to_string()])
        );
        assert_eq!(
            index.get("c/d"),
            Some(&vec!["🔥 In Production".to_string()])
        );
    }

    #[test]
    fn test_build_focus_index_multiple_focus_definition_order() {
        let f1 = StarList {
            name: "Focus: A".to_string(),
            description: None,
            repositories: vec![make_repo("x/y", None)],
        };
        let f2 = StarList {
            name: "Focus: B".to_string(),
            description: None,
            repositories: vec![make_repo("x/y", None)],
        };
        let index = build_focus_index(&[&f1, &f2]);
        assert_eq!(
            index.get("x/y"),
            Some(&vec!["A".to_string(), "B".to_string()])
        );
    }

    #[test]
    fn test_build_focus_index_skips_empty_display() {
        let f = StarList {
            name: "Focus: ".to_string(),
            description: None,
            repositories: vec![make_repo("x/y", None)],
        };
        let index = build_focus_index(&[&f]);
        assert!(index.is_empty());
    }
}
