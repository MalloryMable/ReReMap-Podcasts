pub mod network;
pub mod people;
pub mod rules;
pub mod url_hash;

use chrono::{DateTime, Utc};
use db::queries;
use db::tables::{Episode, Podcast};
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
        .fetch_feed(
            &podcast.rss_url,
            podcast.etag.as_deref(),
            podcast.last_modified.as_deref(),
            podcast.min_episode_length_sec,
        )
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
        let published_at = ep
            .published_at
            .map(|d| DateTime::<Utc>::from_naive_utc_and_offset(d, Utc))
            .unwrap_or_else(Utc::now);

        let episode = Episode {
            id: 0,
            podcast_id: podcast.id,
            guid: ep.guid,
            title: ep.title,
            published_at,
            raw_description: ep.raw_description,
        };

        // NOTE: Existing eps get None might want a more expressive error here
        if let Some(episode_id) = queries::insert_episode(pool, &episode).await? {
            let description = episode.raw_description.as_deref().unwrap_or("");
            people::process_guests(pool, episode_id, &episode.title, description, rules).await?;
        }
    }

    Ok(())
}
//TODO: insert alias


// TODO: insert episode regex
