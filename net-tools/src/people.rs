use std::collections::HashSet;
use sqlx::MySqlPool;
use crate::rules::CompiledRule;
use db::queries::{find_id_by_alias, insert_person, add_appearance};

pub async fn create_person(pool: &MySqlPool, name: &str) -> Result<u64, sqlx::Error> {
    insert_person(pool, name).await
}

pub async fn resolve_alias(pool: &MySqlPool, name: &str) -> Result<u64, sqlx::Error> {
    if let Some(id) = find_id_by_alias(pool, name).await? {
        return Ok(id);
    }

    // TODO: more in depth control flow
    println!("No person matching the alias: {} found.\nAdding new person", name);
    create_person(pool, name).await
}

pub async fn process_episode_guests(
    pool: &MySqlPool,
    episode_id: u64,
    title: &str,
    description: &str,
    rules: &[CompiledRule],
) -> Result<(), sqlx::Error> {
    let mut matched_person_ids: HashSet<u64> = HashSet::new();

    for rule in rules {
        for name in rule.extract_all(title, description) {
            let person_id = resolve_or_create_person(pool, &name).await?;
            matched_person_ids.insert(person_id);
        }
    }

    for person_id in matched_person_ids {
        add_episode_appearance(pool, episode_id, person_id).await?;
    }

    Ok(())
}
