#[cfg(test)]
mod tests {
    use super::queries::*;
    use super::tables::*;
    use sqlx::MySqlPool;
    use chrono::Utc;

    #[sqlx::test]
    async fn test_podcast_insertion_and_fetch(pool: MySqlPool) {
        let podcast_id = insert_podcast(
            &pool,
            "Waypoint Radio",
            "https://example.com/rss",
            "hash123",
        )
        .await
        .expect("Failed to insert podcast");

        let fetched = get_podcast_by_hash(&pool, "hash123")
            .await
            .expect("Failed to query")
            .expect("Podcast not found");

        assert_eq!(fetched.id, podcast_id);
        assert_eq!(fetched.title, "Waypoint Radio");
    }

    #[sqlx::test]
    async fn test_cascading_deletes(pool: MySqlPool) {
        // insert podcast
        let podcast_id = insert_podcast(&pool, "Test Pod", "url", "hash").await.unwrap();

        // insert episode
        let ep = Episode {
        id: 0,
        podcast_id,
        guid: "ep-1".to_string(),
        title: "Episode 1".to_string(),
        published_at: Utc::now(),
        raw_description: None,
};


        let episode_id = insert_episode(&pool, &ep).await.unwrap().expect("Episode was duplicate");

        // delete podcast
        sqlx::query("DELETE FROM podcasts WHERE id = ?")
            .bind(podcast_id)
            .execute(&pool)
            .await
            .unwrap();

        let ep_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM episodes WHERE id = ?")
            .bind(episode_id)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(ep_count.0, 0, "Episode should have been cascaded deleted");
    }
}
