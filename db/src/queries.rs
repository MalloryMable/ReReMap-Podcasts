use sqlx::MySqlPool;

use crate::tables::{Episode, Podcast};

pub async fn get_podcast_by_hash(
    pool: &MySqlPool,
    rss_url_hash: &str,
) -> Result<Option<Podcast>, sqlx::Error> {
    sqlx::query_as::<_, Podcast>("SELECT * FROM podcasts WHERE rss_url_hash = ?")
        .bind(rss_url_hash)
        .fetch_optional(pool)
        .await
}

/// rss_url and rss_url_hash are expected to already be normalized/hashed
/// by net-tools before this is called.
pub async fn insert_podcast(
    pool: &MySqlPool,
    title: &str,
    rss_url: &str,
    rss_url_hash: &str,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO podcasts (title, rss_url, rss_url_hash) VALUES (?, ?, ?)",
    )
    .bind(title)
    .bind(rss_url)
    .bind(rss_url_hash)
    .execute(pool)
    .await?;

    Ok(result.last_insert_id())
}

pub async fn insert_episode(pool: &MySqlPool, ep: &Episode) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO episodes (podcast_id, guid, title, audio_url, published_at, raw_description)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(ep.podcast_id)
    .bind(&ep.guid)
    .bind(&ep.title)
    .bind(&ep.audio_url)
    .bind(ep.published_at)
    .bind(&ep.raw_description)
    .execute(pool)
    .await?;

    Ok(result.last_insert_id())
}

pub async fn add_person_alias(
    pool: &MySqlPool,
    person_id: u64,
    alias: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT IGNORE INTO person_aliases (person_id, alias) VALUES (?, ?)")
        .bind(person_id)
        .bind(alias)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn set_podcast_host(
    pool: &MySqlPool,
    podcast_id: u64,
    person_id: u64,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT IGNORE INTO podcast_hosts (podcast_id, person_id) VALUES (?, ?)")
        .bind(podcast_id)
        .bind(person_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn add_episode_appearance(
    pool: &MySqlPool,
    episode_id: u64,
    person_id: u64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO episode_appearances (episode_id, person_id) VALUES (?, ?)",
    )
    .bind(episode_id)
    .bind(person_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Role is derived, not stored: host if the person is in podcast_hosts
/// for this episode's podcast, guest otherwise.
pub async fn get_episode_roles(
    pool: &MySqlPool,
    episode_id: u64,
) -> Result<Vec<(u64, String)>, sqlx::Error> {
    sqlx::query_as::<_, (u64, String)>(
        "SELECT ea.person_id,
                CASE WHEN ph.person_id IS NOT NULL THEN 'host' ELSE 'guest' END
         FROM episode_appearances ea
         JOIN episodes e ON e.id = ea.episode_id
         LEFT JOIN podcast_hosts ph
           ON ph.podcast_id = e.podcast_id AND ph.person_id = ea.person_id
         WHERE ea.episode_id = ?",
    )
    .bind(episode_id)
    .fetch_all(pool)
    .await
}

pub async fn find_person_id_by_name_or_alias(
    pool: &MySqlPool,
    name: &str,
) -> Result<Option<u64>, sqlx::Error> {
    sqlx::query_scalar::<_, u64>(
        "SELECT id FROM people WHERE name = ?
         UNION
         SELECT person_id FROM person_aliases WHERE alias = ?
         LIMIT 1",
    )
    .bind(name)
    .bind(name)
    .fetch_optional(pool)
    .await
}

pub async fn insert_person(pool: &MySqlPool, name: &str) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("INSERT INTO people (name) VALUES (?)")
        .bind(name)
        .execute(pool)
        .await?;
    Ok(result.last_insert_id())
}

pub async fn update_podcast_url(
    pool: &MySqlPool,
    podcast_id: u64,
    new_rss_url: &str,
    new_rss_url_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE podcasts SET rss_url = ?, rss_url_hash = ? WHERE id = ?")
        .bind(new_rss_url)
        .bind(new_rss_url_hash)
        .bind(podcast_id)
        .execute(pool)
        .await?;
    Ok(())
}
