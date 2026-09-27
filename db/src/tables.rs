use chrono::{DateTime, Utc};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Person {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PersonAlias {
    pub person_id: u64,
    pub alias: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Podcast {
    pub id: u64,
    pub title: String,
    pub rss_url: String,
    pub rss_url_hash: String,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub min_episode_length_sec: u32,
    pub is_active: bool,
    pub last_checked_at: Option<DateTime<Utc>>,
    pub last_success_at: Option<DateTime<Utc>>,
    pub consecutive_failures: u32,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Episode {
    pub id: u64,
    pub podcast_id: u64,
    pub guid: String,
    pub title: String,
    pub published_at: DateTime<Utc>,
    pub raw_description: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EpisodeAppearance {
    pub episode_id: u64,
    pub person_id: u64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PodcastHost {
    pub podcast_id: u64,
    pub person_id: u64,
}


// Tables for parsing episodes for guests
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum RuleTargetField {
    Title,
    Description,
}

/// Row from `parsing_rules`. `podcast_id: None` means a global fallback rule
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FeedParsingRuleRow {
    pub id: u64,
    pub podcast_id: Option<u64>,
    pub target_field: RuleTargetField,
    pub pattern: String,
    pub capture_group: i32,
    pub priority: i32,
}
