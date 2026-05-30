use std::collections::{HashMap, HashSet};

use crate::github::types::{Repository, StarList};

/// Generate an Awesome List formatted Markdown from Star Lists and all starred repos.
///
/// `lists` — Star Lists (each containing its repositories)
/// `all_starred` — All starred repositories (regardless of list membership)
///
/// Stars not belonging to any list are grouped under an "Uncategorized" section.
pub fn generate(
    lists: &[StarList],
    all_starred: &[Repository],
    config: &crate::config::Config,
) -> String {
    let mut out = String::new();

    // Header
    out.push_str("# Awesome Stars\n\n");
    out.push_str("> A curated list of my GitHub stars, organized by lists.\n\n");

    // Partition: Focus Lists are tagged inline, topic lists become sections.
    let (focus_lists, topic_lists): (Vec<&StarList>, Vec<&StarList>) =
        lists.iter().partition(|l| is_focus_list(&l.name));
    let focus_index = build_focus_index(&focus_lists);

    // Apply config.order to topic lists.
    let topic_lists = apply_order(topic_lists, &config.order);

    // Collect valid focus lists (those with a non-empty display name).
    // Emit a warning on stderr for invalid ones.
    let valid_focus: Vec<(String, &StarList)> = focus_lists
        .iter()
        .filter_map(|l| {
            let display = focus_display_name(&l.name);
            if strip_emoji_prefix(&display).trim().is_empty() {
                eprintln!(
                    "warning: Focus List with empty display name skipped: {:?}",
                    l.name
                );
                None
            } else {
                Some((display, *l))
            }
        })
        .collect();

    if !valid_focus.is_empty() {
        out.push_str("## Focus\n\n");
        for (display, list) in &valid_focus {
            let desc = list.description.as_deref().unwrap_or("");
            if desc.is_empty() {
                out.push_str(&format!("- `{display}`\n"));
            } else {
                out.push_str(&format!("- `{display}` — {desc}\n"));
            }
        }
        out.push('\n');
    }

    // Uncategorized = repos that don't belong to any TOPIC list.
    // A repo that lives only in Focus Lists ends up here (with its Focus tags from Task 6).
    let listed_repos: HashSet<&str> = topic_lists
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

    // Contents (TOC) — topic lists only
    out.push_str("## Contents\n\n");
    for list in &topic_lists {
        let anchor = to_anchor(&list.name);
        out.push_str(&format!("- [{}](#{})\n", list.name, anchor));
    }
    if !uncategorized.is_empty() {
        out.push_str("- [Uncategorized](#uncategorized)\n");
    }
    out.push('\n');

    // Topic sections only
    for list in &topic_lists {
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
            let tags = focus_index
                .get(&repo.name_with_owner)
                .cloned()
                .unwrap_or_default();
            write_repo_line(&mut out, repo, &tags);
        }
        out.push('\n');
    }

    // Uncategorized section
    if !uncategorized.is_empty() {
        out.push_str("## Uncategorized\n\n");
        for repo in &uncategorized {
            let tags = focus_index
                .get(&repo.name_with_owner)
                .cloned()
                .unwrap_or_default();
            write_repo_line(&mut out, repo, &tags);
        }
        out.push('\n');
    }

    out
}

fn write_repo_line(out: &mut String, repo: &Repository, focus_tags: &[String]) {
    let desc = repo
        .description
        .as_deref()
        .unwrap_or("No description provided");
    out.push_str(&format!(
        "- [{}]({}) - {}",
        repo.name_with_owner, repo.url, desc
    ));
    for tag in focus_tags {
        out.push_str(&format!(" `{tag}`"));
    }
    out.push('\n');
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
fn build_focus_index(focus_lists: &[&StarList]) -> HashMap<String, Vec<String>> {
    let mut index: HashMap<String, Vec<String>> = HashMap::new();
    for list in focus_lists {
        let display = focus_display_name(&list.name);
        if strip_emoji_prefix(&display).trim().is_empty() {
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

/// Reorder `topic_lists` according to `order`.
/// Lists named in `order` appear first (in that order); remaining lists are
/// appended at the end in their original input order, with a stderr warning.
fn apply_order<'a>(topic_lists: Vec<&'a StarList>, order: &[String]) -> Vec<&'a StarList> {
    if order.is_empty() {
        return topic_lists;
    }
    let mut remaining: HashMap<&str, &StarList> =
        topic_lists.iter().map(|l| (l.name.as_str(), *l)).collect();
    let mut ordered: Vec<&StarList> = Vec::with_capacity(topic_lists.len());
    for name in order {
        if let Some(l) = remaining.remove(name.as_str()) {
            ordered.push(l);
        }
    }
    // Warn for lists present on GitHub but missing from config.
    for list in &topic_lists {
        if remaining.contains_key(list.name.as_str()) {
            eprintln!(
                "warning: list \"{}\" is not in starmap.toml order; appended at end",
                list.name
            );
        }
    }
    // Append remaining (preserving original order).
    for list in topic_lists {
        if remaining.contains_key(list.name.as_str()) {
            ordered.push(list);
        }
    }
    ordered
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
            stargazer_count: None,
            language: None,
            topics: vec![],
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

        let md = generate(&lists, &all_starred, &crate::config::Config::default());

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

        let md = generate(&lists, &all_starred, &crate::config::Config::default());

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

        let md = generate(&lists, &[], &crate::config::Config::default());
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
        // After stripping, only "emoji + space" remains. The validity gate strips the
        // emoji prefix again to detect this as empty.
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

    #[test]
    fn test_write_repo_line_no_tags() {
        let mut out = String::new();
        let repo = make_repo("a/b", Some("desc"));
        write_repo_line(&mut out, &repo, &[]);
        assert_eq!(out, "- [a/b](https://github.com/a/b) - desc\n");
    }

    #[test]
    fn test_write_repo_line_with_tags() {
        let mut out = String::new();
        let repo = make_repo("a/b", Some("desc"));
        let tags = vec!["🔥 In Production".to_string(), "🌱 Watching".to_string()];
        write_repo_line(&mut out, &repo, &tags);
        assert_eq!(
            out,
            "- [a/b](https://github.com/a/b) - desc `🔥 In Production` `🌱 Watching`\n"
        );
    }

    #[test]
    fn test_generate_tags_appear_on_repo_lines_in_topic_section() {
        let lists = vec![
            StarList {
                name: "🤖 AI Frameworks".to_string(),
                description: None,
                repositories: vec![make_repo("a/b", Some("desc"))],
            },
            StarList {
                name: "🔥 Focus: In Production".to_string(),
                description: None,
                repositories: vec![make_repo("a/b", None)],
            },
        ];
        let md = generate(
            &lists,
            &[make_repo("a/b", Some("desc"))],
            &crate::config::Config::default(),
        );
        assert!(md.contains("- [a/b](https://github.com/a/b) - desc `🔥 In Production`"));
    }

    #[test]
    fn test_generate_tags_appear_on_uncategorized_repos() {
        let lists = vec![StarList {
            name: "🔥 Focus: In Production".to_string(),
            description: None,
            repositories: vec![make_repo("orphan/repo", None)],
        }];
        let all = vec![make_repo("orphan/repo", Some("orphan desc"))];
        let md = generate(&lists, &all, &crate::config::Config::default());

        // orphan/repo is in a Focus List but no topic List → Uncategorized with tag
        assert!(md.contains("## Uncategorized"));
        assert!(md.contains(
            "- [orphan/repo](https://github.com/orphan/repo) - orphan desc `🔥 In Production`"
        ),);
    }

    #[test]
    fn test_generate_excludes_focus_from_toc_and_sections() {
        let lists = vec![
            StarList {
                name: "🤖 AI Frameworks".to_string(),
                description: None,
                repositories: vec![make_repo("anthropics/sdk", Some("SDK"))],
            },
            StarList {
                name: "🔥 Focus: In Production".to_string(),
                description: None,
                repositories: vec![make_repo("anthropics/sdk", None)],
            },
        ];
        let all = vec![make_repo("anthropics/sdk", Some("SDK"))];
        let md = generate(&lists, &all, &crate::config::Config::default());

        // Focus List must NOT appear in TOC
        assert!(!md.contains("- [🔥 Focus: In Production]"));
        // Focus List must NOT have its own section header
        assert!(!md.contains("## 🔥 Focus: In Production"));
        // Topic List section is still rendered
        assert!(md.contains("## 🤖 AI Frameworks"));
    }

    #[test]
    fn test_generate_emits_focus_legend_with_description() {
        let lists = vec![StarList {
            name: "🔥 Focus: In Production".to_string(),
            description: Some("業務で使用中".to_string()),
            repositories: vec![],
        }];
        let md = generate(&lists, &[], &crate::config::Config::default());
        assert!(md.contains("## Focus"));
        assert!(md.contains("- `🔥 In Production` — 業務で使用中"));
    }

    #[test]
    fn test_generate_focus_legend_omits_dash_when_description_empty() {
        let lists = vec![StarList {
            name: "🔥 Focus: In Production".to_string(),
            description: None,
            repositories: vec![],
        }];
        let md = generate(&lists, &[], &crate::config::Config::default());
        assert!(md.contains("- `🔥 In Production`\n"));
        assert!(!md.contains("- `🔥 In Production` —"));
    }

    #[test]
    fn test_generate_no_focus_legend_when_zero_focus_lists() {
        let lists = vec![StarList {
            name: "🤖 AI Frameworks".to_string(),
            description: None,
            repositories: vec![],
        }];
        let md = generate(&lists, &[], &crate::config::Config::default());
        assert!(!md.contains("## Focus\n"));
    }

    #[test]
    fn test_build_focus_index_skips_emoji_only_bare_display() {
        let f = StarList {
            name: "🔥 Focus: ".to_string(),
            description: None,
            repositories: vec![make_repo("a/b", None)],
        };
        let index = build_focus_index(&[&f]);
        assert!(
            index.is_empty(),
            "emoji-only bare Focus List should be skipped"
        );
    }

    #[test]
    fn test_generate_skips_emoji_only_bare_focus_list() {
        let lists = vec![StarList {
            name: "🔥 Focus: ".to_string(),
            description: Some("ignored".to_string()),
            repositories: vec![make_repo("a/b", Some("desc"))],
        }];
        let md = generate(
            &lists,
            &[make_repo("a/b", Some("desc"))],
            &crate::config::Config::default(),
        );

        // No legend
        assert!(!md.contains("## Focus\n"));
        // No emoji-only tag rendered anywhere
        assert!(!md.contains("`🔥 `"));
        // Repo lands in Uncategorized with no tag
        assert!(md.contains("## Uncategorized"));
        assert!(md.contains("- [a/b](https://github.com/a/b) - desc\n"));
    }

    #[test]
    fn test_generate_skips_focus_list_with_empty_display_name() {
        let lists = vec![StarList {
            name: "Focus: ".to_string(), // bare prefix, no display name
            description: None,
            repositories: vec![make_repo("a/b", Some("desc"))],
        }];
        let md = generate(
            &lists,
            &[make_repo("a/b", Some("desc"))],
            &crate::config::Config::default(),
        );

        // No legend section is emitted (no valid focus entries)
        assert!(!md.contains("## Focus\n"));
        // The repo lands in Uncategorized without a tag (the bad list contributed nothing)
        assert!(md.contains("## Uncategorized"));
        assert!(md.contains("- [a/b](https://github.com/a/b) - desc\n"));
    }

    #[test]
    fn generate_respects_config_order() {
        let lists = vec![
            StarList {
                name: "B".into(),
                description: None,
                repositories: vec![],
            },
            StarList {
                name: "A".into(),
                description: None,
                repositories: vec![],
            },
            StarList {
                name: "C".into(),
                description: None,
                repositories: vec![],
            },
        ];
        let cfg = crate::config::Config {
            order: vec!["A".into(), "C".into()],
            ..Default::default()
        };
        let out = generate(&lists, &[], &cfg);
        // A first, C second, B (not in config) appended last.
        // Use "\n## X\n" to avoid matching "## Contents" for the "C" case.
        let pos_a = out.find("\n## A\n").unwrap();
        let pos_c = out.find("\n## C\n").unwrap();
        let pos_b = out.find("\n## B\n").unwrap();
        assert!(pos_a < pos_c && pos_c < pos_b);
    }

    #[test]
    fn generate_no_config_keeps_input_order() {
        let lists = vec![
            StarList {
                name: "First".into(),
                description: None,
                repositories: vec![],
            },
            StarList {
                name: "Second".into(),
                description: None,
                repositories: vec![],
            },
        ];
        let out = generate(&lists, &[], &crate::config::Config::default());
        let pos_first = out.find("## First").unwrap();
        let pos_second = out.find("## Second").unwrap();
        assert!(pos_first < pos_second);
    }

    #[test]
    fn test_generate_full_flow_with_focus() {
        let lists = vec![
            StarList {
                name: "🤖 AI Frameworks".to_string(),
                description: Some("LLM SDKs and frameworks".to_string()),
                repositories: vec![
                    make_repo("anthropics/anthropic-sdk-python", Some("Anthropic SDK")),
                    make_repo("langchain-ai/langgraph", Some("Resilient agents")),
                ],
            },
            StarList {
                name: "🔥 Focus: In Production".to_string(),
                description: Some("業務 / 個人開発で実際に使ってる".to_string()),
                repositories: vec![make_repo("anthropics/anthropic-sdk-python", None)],
            },
            StarList {
                name: "🌱 Focus: Watching".to_string(),
                description: Some("試したい・気になってる".to_string()),
                repositories: vec![
                    make_repo("anthropics/anthropic-sdk-python", None),
                    make_repo("langchain-ai/langgraph", None),
                ],
            },
        ];
        let all = vec![
            make_repo("anthropics/anthropic-sdk-python", Some("Anthropic SDK")),
            make_repo("langchain-ai/langgraph", Some("Resilient agents")),
            make_repo("orphan/repo", Some("Lonely")),
        ];

        let md = generate(&lists, &all, &crate::config::Config::default());

        // Legend
        assert!(md.contains("## Focus\n"));
        assert!(md.contains("- `🔥 In Production` — 業務 / 個人開発で実際に使ってる"));
        assert!(md.contains("- `🌱 Watching` — 試したい・気になってる"));

        // TOC excludes Focus
        assert!(md.contains("- [🤖 AI Frameworks]"));
        assert!(!md.contains("- [🔥 Focus: In Production]"));
        assert!(!md.contains("- [🌱 Focus: Watching]"));
        assert!(md.contains("- [Uncategorized](#uncategorized)"));

        // Topic section
        assert!(md.contains("## 🤖 AI Frameworks"));
        assert!(md.contains("> LLM SDKs and frameworks"));
        // anthropics/anthropic-sdk-python: in BOTH focus lists → tags in definition order
        assert!(md.contains(
            "- [anthropics/anthropic-sdk-python](https://github.com/anthropics/anthropic-sdk-python) - Anthropic SDK `🔥 In Production` `🌱 Watching`"
        ));
        // langchain-ai/langgraph: in Watching only
        assert!(md.contains(
            "- [langchain-ai/langgraph](https://github.com/langchain-ai/langgraph) - Resilient agents `🌱 Watching`"
        ));

        // No focus section header
        assert!(!md.contains("## 🔥 Focus: In Production"));
        assert!(!md.contains("## 🌱 Focus: Watching"));

        // Uncategorized
        assert!(md.contains("## Uncategorized"));
        assert!(md.contains("- [orphan/repo](https://github.com/orphan/repo) - Lonely\n"));
    }
}
