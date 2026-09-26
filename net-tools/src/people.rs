use db::queries::{find_person_id_by_name_or_alias, insert_person};
use sqlx::MySqlPool;

/// Pure read against people.name / person_aliases.alias. Never creates
/// anything, never logs -- returns None on a miss and lets the caller decide.
pub async fn resolve_alias(pool: &MySqlPool, name: &str) -> Result<Option<u64>, sqlx::Error> {
    find_person_id_by_name_or_alias(pool, name).await
}

/// Pure write -- no lookup, no logging. Safe to call directly (bulk import,
/// tests, a future "yes, definitely a new person" confirmation) without
/// triggering side effects meant for the auto-ingest path.
pub async fn create_person(pool: &MySqlPool, name: &str) -> Result<u64, sqlx::Error> {
    insert_person(pool, name).await
}

fn no_matching_name(name: &str) {
    println!("No person matching the alias: {} found.\nAdding new person", name);
}

/// Today's control flow: resolve, and on a miss, log + create. This is the
/// seam that grows a real decision tree later (reject / prompt-to-remap /
/// queue for review) -- resolve_alias and create_person shouldn't need to
/// change when that happens, only this function's body.
pub async fn resolve_or_create_person(pool: &MySqlPool, name: &str) -> Result<u64, sqlx::Error> {
    if let Some(id) = resolve_alias(pool, name).await? {
        return Ok(id);
    }

    no_matching_name(name);
    create_person(pool, name).await
}
