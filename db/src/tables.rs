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
    pub audio_url: Option<String>,
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
