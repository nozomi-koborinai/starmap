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
    #[allow(dead_code)]
    pub stargazer_count: Option<u64>,
    #[allow(dead_code)]
    pub language: Option<String>,
    #[allow(dead_code)]
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
    pub nodes: Vec<RawRepository>,
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
    pub nodes: Vec<RawRepository>,
    pub page_info: PageInfo,
}

// --- Shared types ---

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawRepository {
    pub name_with_owner: String,
    pub description: Option<String>,
    pub url: String,
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
            repositories: raw.items.nodes.into_iter().map(Repository::from).collect(),
        }
    }
}
