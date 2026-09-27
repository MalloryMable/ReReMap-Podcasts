use chrono::Utc;
use db::queries::*;
use db::tables::*;
use sqlx::MySqlPool;

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
    let podcast_id = insert_podcast(&pool, "Test Pod", "url", "hash")
        .await
        .unwrap();

    let ep = Episode {
        id: 0,
        podcast_id,
        guid: "ep-1".to_string(),
        title: "Episode 1".to_string(),
        published_at: Utc::now(),
        raw_description: None,
    };

    let episode_id = insert_episode(&pool, &ep)
        .await
        .unwrap()
        .expect("Episode was duplicate");

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

#[sqlx::test]
async fn test_mark_check_success_resets_failures(pool: MySqlPool) {
    let podcast_id = insert_podcast(&pool, "Flaky Pod", "url", "hash")
        .await
        .unwrap();

    // simulate a few prior failures
    mark_check_failure(&pool, podcast_id, 10).await.unwrap();
    mark_check_failure(&pool, podcast_id, 10).await.unwrap();

    mark_check_success(&pool, podcast_id).await.unwrap();

    let podcast = get_podcast_by_hash(&pool, "hash")
        .await
        .unwrap()
        .expect("podcast not found");

    assert_eq!(podcast.consecutive_failures, 0);
    assert!(podcast.last_checked_at.is_some());
    assert!(podcast.last_success_at.is_some());
    assert!(podcast.is_active, "success should not deactivate a podcast");
}

#[sqlx::test]
async fn test_mark_check_failure_deactivates_after_threshold(pool: MySqlPool) {
    let podcast_id = insert_podcast(&pool, "Dead Pod", "url", "hash")
        .await
        .unwrap();

    mark_check_failure(&pool, podcast_id, 3).await.unwrap();
    mark_check_failure(&pool, podcast_id, 3).await.unwrap();

    let still_active = get_podcast_by_hash(&pool, "hash")
        .await
        .unwrap()
        .expect("podcast not found");
    assert_eq!(still_active.consecutive_failures, 2);
    assert!(still_active.is_active, "shouldn't deactivate before hitting the threshold");

    mark_check_failure(&pool, podcast_id, 3).await.unwrap();

    let now_inactive = get_podcast_by_hash(&pool, "hash")
        .await
        .unwrap()
        .expect("podcast not found");
    assert_eq!(now_inactive.consecutive_failures, 3);
    assert!(!now_inactive.is_active, "should deactivate once threshold is hit");
}

#[sqlx::test]
// TODO: plug these into apis once I have structure queiries.rs
async fn test_get_rules_for_podcast_orders_specific_before_global(pool: MySqlPool) {
    let podcast_id = insert_podcast(&pool, "Rules Pod", "url", "hash")
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO parsing_rules (podcast_id, target_field, pattern, capture_group, priority)
         VALUES (NULL, 'description', 'global-([A-Za-z]+)', 1, 100)",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO parsing_rules (podcast_id, target_field, pattern, capture_group, priority)
         VALUES (?, 'title', 'specific-([A-Za-z]+)', 1, 1)",
    )
    .bind(podcast_id)
    .execute(&pool)
    .await
    .unwrap();

    let rules = get_parsing_rules(&pool, podcast_id).await.unwrap();

    assert_eq!(rules.len(), 2);
    assert_eq!(
        rules[0].podcast_id,
        Some(podcast_id),
        "podcast-specific rule should sort before the global fallback"
    );
    assert_eq!(rules[1].podcast_id, None);
}
