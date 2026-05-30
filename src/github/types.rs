use serde::Deserialize;

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct StarList {
    pub name: String,
    pub description: Option<String>,
    pub repositories: Vec<Repository>,
}

#[derive(Debug, Clone)]
pub struct Repository {
    pub name_with_owner: String,
    pub description: Option<String>,
    pub url: String,
    pub stargazer_count: Option<u64>,
    pub language: Option<String>,
    pub topics: Vec<String>,
}

// ---------------------------------------------------------------------------
// GraphQL raw response types
// ---------------------------------------------------------------------------

// --- viewer.lists query ---

#[derive(Debug, Deserialize)]
pub struct ListsQueryData {
    pub viewer: ListsViewer,
}

#[derive(Debug, Deserialize)]
pub struct ListsViewer {
    pub lists: ListsConnection,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListsConnection {
    pub nodes: Vec<RawUserList>,
    pub page_info: PageInfo,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawUserList {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub items: ListItemsConnection,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListItemsConnection {
    /// Entries are Option because GitHub returns null for repos the viewer
    /// cannot access (private repos in restricted orgs, deleted repos, etc.).
    pub nodes: Vec<Option<RawRepository>>,
    pub page_info: PageInfo,
}

// --- node(id) query (paginated list items) ---

#[derive(Debug, Deserialize)]
pub struct ListItemsQueryData {
    pub node: RawUserListNode,
}

#[derive(Debug, Deserialize)]
pub struct RawUserListNode {
    pub items: ListItemsConnection,
}

// --- viewer.starredRepositories query ---

#[derive(Debug, Deserialize)]
pub struct StarredQueryData {
    pub viewer: StarredViewer,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StarredViewer {
    pub starred_repositories: StarredConnection,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StarredConnection {
    /// Entries are Option for the same reason as ListItemsConnection.nodes.
    pub nodes: Vec<Option<RawRepository>>,
    pub page_info: PageInfo,
}

// --- Shared types ---

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawRepository {
    pub name_with_owner: String,
    pub description: Option<String>,
    pub url: String,
    /// True for private repositories. Filtered out before reaching domain
    /// types so private repo names/contents never appear in published output.
    /// Defaults to false to avoid silently dropping when the API omits the
    /// field (shouldn't happen since we explicitly request it).
    #[serde(default)]
    pub is_private: bool,
    #[serde(default)]
    pub stargazer_count: Option<u64>,
    #[serde(default)]
    pub primary_language: Option<RawLanguage>,
    #[serde(default)]
    pub repository_topics: Option<RawTopics>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawLanguage {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawTopics {
    pub nodes: Vec<RawTopicNode>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawTopicNode {
    pub topic: RawTopicName,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawTopicName {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

// ---------------------------------------------------------------------------
// Raw -> Domain conversion
// ---------------------------------------------------------------------------

impl From<RawRepository> for Repository {
    fn from(raw: RawRepository) -> Self {
        Self {
            name_with_owner: raw.name_with_owner,
            description: raw.description,
            url: raw.url,
            stargazer_count: raw.stargazer_count,
            language: raw.primary_language.map(|l| l.name),
            topics: raw
                .repository_topics
                .map(|t| t.nodes.into_iter().map(|n| n.topic.name).collect())
                .unwrap_or_default(),
        }
    }
}

impl From<RawUserList> for StarList {
    fn from(raw: RawUserList) -> Self {
        Self {
            name: raw.name,
            description: raw.description,
            repositories: raw
                .items
                .nodes
                .into_iter()
                .flatten()
                .filter(|r| !r.is_private)
                .map(Repository::from)
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression test: GitHub returns null for repos the viewer cannot access
    /// (private repos in restricted orgs, deleted repos). We must tolerate
    /// nulls in nodes and skip them.
    #[test]
    fn list_items_connection_tolerates_null_nodes() {
        let json = r#"{
            "nodes": [
                {"nameWithOwner": "a/b", "description": null, "url": "https://github.com/a/b"},
                null,
                {"nameWithOwner": "c/d", "description": "ok", "url": "https://github.com/c/d"}
            ],
            "pageInfo": {"hasNextPage": false, "endCursor": null}
        }"#;
        let conn: ListItemsConnection = serde_json::from_str(json).unwrap();
        assert_eq!(conn.nodes.len(), 3);
        let repos: Vec<Repository> = conn
            .nodes
            .into_iter()
            .flatten()
            .map(Repository::from)
            .collect();
        assert_eq!(repos.len(), 2);
        assert_eq!(repos[0].name_with_owner, "a/b");
        assert_eq!(repos[1].name_with_owner, "c/d");
    }

    /// Regression: private repos must be filtered out when converting
    /// RawUserList to StarList so their names never reach the published
    /// awesome list.
    #[test]
    fn star_list_drops_private_repos() {
        let json = r#"{
            "id": "L1",
            "name": "Mixed",
            "description": null,
            "items": {
                "nodes": [
                    {"nameWithOwner": "public/a", "description": null, "url": "u", "isPrivate": false},
                    {"nameWithOwner": "secret/b", "description": null, "url": "u", "isPrivate": true},
                    {"nameWithOwner": "public/c", "description": null, "url": "u", "isPrivate": false}
                ],
                "pageInfo": {"hasNextPage": false, "endCursor": null}
            }
        }"#;
        let raw: RawUserList = serde_json::from_str(json).unwrap();
        let star_list: StarList = raw.into();
        let names: Vec<_> = star_list
            .repositories
            .iter()
            .map(|r| r.name_with_owner.as_str())
            .collect();
        assert_eq!(names, vec!["public/a", "public/c"]);
    }

    #[test]
    fn starred_connection_tolerates_null_nodes() {
        let json = r#"{
            "nodes": [
                null,
                {"nameWithOwner": "a/b", "description": null, "url": "https://github.com/a/b"}
            ],
            "pageInfo": {"hasNextPage": false, "endCursor": null}
        }"#;
        let conn: StarredConnection = serde_json::from_str(json).unwrap();
        let repos: Vec<Repository> = conn
            .nodes
            .into_iter()
            .flatten()
            .map(Repository::from)
            .collect();
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].name_with_owner, "a/b");
    }
}
