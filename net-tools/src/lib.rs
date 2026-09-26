pub mod network;
pub mod people;
pub mod rules;
pub mod url_hash;

use std::collections::HashSet;

use chrono::{DateTime, Utc};
use db::queries;
use db::tables::Podcast;
use network::{FetchResult, NetworkModel};
use rules::CompiledRule;
use sqlx::MySqlPool;

pub async fn ingest_podcast(
    pool: &MySqlPool,
    net: &NetworkModel,
    podcast: &Podcast,
    rules: &[CompiledRule],
) -> Result<(), Box<dyn std::error::Error>> {
    let fetch = net
        .fetch_feed(&podcast.rss_url, podcast.etag.as_deref(), podcast.last_modified.as_deref())
        .await;

    let feed = match fetch {
        FetchResult::NotModified => return Ok(()),
        FetchResult::Error(e) => return Err(e.into()),
        FetchResult::Success(feed) => feed,
    };

    if feed.final_url != podcast.rss_url {
        let normalized = url_hash::normalize_feed_url(&feed.final_url);
        let hash = url_hash::hash_url(&normalized);
        queries::update_podcast_url(pool, podcast.id, &normalized, &hash).await?;
    }

    for ep in feed.episodes {
        if podcast.min_episode_length_sec > 0 {
            match ep.duration_sec {
                Some(d) if d < podcast.min_episode_length_sec => continue,
                None => continue, // filter's on, length unconfirmed -> skip
                _ => {}
            }
        }

        let published_at = ep
            .published_at
            .map(|d| DateTime::<Utc>::from_naive_utc_and_offset(d, Utc))
            .unwrap_or_else(Utc::now);

        let episode_id = queries::insert_episode(
            pool, podcast.id, &ep.guid, &ep.title, Some(&ep.audio_url),
            published_at, ep.raw_description.as_deref(),
        ).await?;

        let description = ep.raw_description.as_deref().unwrap_or("");
        let mut matched_person_ids: HashSet<u64> = HashSet::new();

        for rule in rules {
            for name in rule.extract_all(&ep.title, description) {
                let person_id = people::resolve_or_create_person(pool, &name).await?;
                matched_person_ids.insert(person_id);
            }
        }

    }

    Ok(())
}
