use crate::config::Config;
use crate::github::types::{Repository, StarList};

/// Generate llmstxt.org-compliant index of starred repos.
#[allow(dead_code)]
pub fn generate(title: &str, lists: &[StarList], config: &Config) -> String {
    let mut out = String::new();
    out.push_str(&format!("# {title}\n\n"));
    out.push_str("> Personally curated GitHub stars\n\n");

    // Apply config order; lists not in config appended at end.
    let ordered = apply_order(lists, &config.order);

    for list in ordered {
        out.push_str(&format!("## {}\n\n", list.name));
        let mut repos: Vec<&Repository> = list.repositories.iter().collect();
        repos.sort_by(|a, b| {
            a.name_with_owner
                .to_lowercase()
                .cmp(&b.name_with_owner.to_lowercase())
        });
        for r in repos {
            let desc = r
                .description
                .as_deref()
                .unwrap_or("")
                .replace(['\n', '\r'], " ");
            if desc.is_empty() {
                out.push_str(&format!("- [{}]({})\n", r.name_with_owner, r.url));
            } else {
                out.push_str(&format!("- [{}]({}): {}\n", r.name_with_owner, r.url, desc));
            }
        }
        out.push('\n');
    }
    out
}

fn apply_order<'a>(lists: &'a [StarList], order: &[String]) -> Vec<&'a StarList> {
    if order.is_empty() {
        return lists.iter().collect();
    }
    let mut remaining: std::collections::HashMap<&str, &StarList> =
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
    use crate::github::types::Repository;

    fn repo(nwo: &str, desc: Option<&str>) -> Repository {
        Repository {
            name_with_owner: nwo.into(),
            description: desc.map(str::to_string),
            url: format!("https://github.com/{nwo}"),
            stargazer_count: None,
            language: None,
            topics: vec![],
        }
    }

    #[test]
    fn generates_header_and_sections() {
        let lists = vec![StarList {
            name: "🤖 AI".into(),
            description: None,
            repositories: vec![repo("a/b", Some("hello"))],
        }];
        let out = generate("user stars", &lists, &Config::default());
        assert!(out.starts_with("# user stars\n"));
        assert!(out.contains("> Personally curated GitHub stars"));
        assert!(out.contains("## 🤖 AI"));
        assert!(out.contains("- [a/b](https://github.com/a/b): hello"));
    }

    #[test]
    fn description_newlines_collapsed() {
        let lists = vec![StarList {
            name: "X".into(),
            description: None,
            repositories: vec![repo("a/b", Some("line1\nline2"))],
        }];
        let out = generate("t", &lists, &Config::default());
        assert!(out.contains("line1 line2"));
        assert!(!out.contains("line1\nline2"));
    }

    #[test]
    fn empty_description_omits_colon() {
        let lists = vec![StarList {
            name: "X".into(),
            description: None,
            repositories: vec![repo("a/b", None)],
        }];
        let out = generate("t", &lists, &Config::default());
        assert!(out.contains("- [a/b](https://github.com/a/b)\n"));
        assert!(!out.contains("- [a/b](https://github.com/a/b):"));
    }

    #[test]
    fn order_applied() {
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
        ];
        let cfg = Config {
            order: vec!["A".into(), "B".into()],
            ..Default::default()
        };
        let out = generate("t", &lists, &cfg);
        assert!(out.find("## A").unwrap() < out.find("## B").unwrap());
    }
}
