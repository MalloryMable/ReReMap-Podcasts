-- TODO: ledger, don't just rebuild from scratch. (post hackaton)

-- NOTE: mariadb --socket="$MYSQL_SOCK" "$MARIADB_DATABASE" < db/schema.sql

DROP TABLE IF EXISTS episode_appearances;
DROP TABLE IF EXISTS podcast_hosts;
DROP TABLE IF EXISTS episodes;
DROP TABLE IF EXISTS person_aliases;
DROP TABLE IF EXISTS podcasts;
DROP TABLE IF EXISTS people;
DROP TABLE IF EXISTS parsing_rules;

-- ENGINE/CHARSET/COLLATE inherited from the database default

CREATE TABLE people (
    id    BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    name  VARCHAR(255) NOT NULL   -- canonical display name
);

CREATE TABLE person_aliases (
    person_id  BIGINT UNSIGNED NOT NULL,
    alias      VARCHAR(255) NOT NULL,
    PRIMARY KEY (person_id, alias),
    CONSTRAINT fk_aliases_person FOREIGN KEY (person_id) REFERENCES people(id) ON DELETE CASCADE
);

CREATE TABLE podcasts (
    id                       BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    title                    VARCHAR(512) NOT NULL,
    rss_url                  TEXT NOT NULL,
    rss_url_hash             CHAR(64) NOT NULL,   -- computed in net-tools
    etag                     VARCHAR(255) NULL,
    last_modified            VARCHAR(255) NULL,
    min_episode_length_sec   INT UNSIGNED NOT NULL DEFAULT 0,  -- 0 = filter disabled
    is_active                BOOLEAN NOT NULL DEFAULT TRUE,
    last_checked_at          DATETIME NULL,
    last_success_at          DATETIME NULL,
    consecutive_failures     INT UNSIGNED NOT NULL DEFAULT 0,
    UNIQUE KEY uq_podcasts_rss_url_hash (rss_url_hash)
);

CREATE TABLE episodes (
    id               BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    podcast_id       BIGINT UNSIGNED NOT NULL,
    guid             VARCHAR(512) NOT NULL,
    title            VARCHAR(512) NOT NULL,
    published_at     DATETIME NOT NULL,
    raw_description  TEXT NULL,
    CONSTRAINT fk_episodes_podcast FOREIGN KEY (podcast_id) REFERENCES podcasts(id) ON DELETE CASCADE,
    UNIQUE KEY uq_episodes_podcast_guid (podcast_id, guid),
    INDEX idx_episodes_published_at (published_at)
);

CREATE TABLE podcast_hosts (
    podcast_id  BIGINT UNSIGNED NOT NULL,
    person_id   BIGINT UNSIGNED NOT NULL,
    PRIMARY KEY (podcast_id, person_id),
    CONSTRAINT fk_podcast_hosts_podcast FOREIGN KEY (podcast_id) REFERENCES podcasts(id) ON DELETE CASCADE,
    CONSTRAINT fk_podcast_hosts_person FOREIGN KEY (person_id) REFERENCES people(id) ON DELETE CASCADE
);

CREATE TABLE episode_appearances (
    episode_id  BIGINT UNSIGNED NOT NULL,
    person_id   BIGINT UNSIGNED NOT NULL,
    -- TODO: confidence  ENUM('certain', 'likely', 'low') NOT NULL DEFAULT 'certain',
    PRIMARY KEY (episode_id, person_id),
    CONSTRAINT fk_appearances_episode FOREIGN KEY (episode_id) REFERENCES episodes(id) ON DELETE CASCADE,
    CONSTRAINT fk_appearances_person FOREIGN KEY (person_id) REFERENCES people(id) ON DELETE CASCADE
);

CREATE TABLE parsing_rules (
    id             BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    podcast_id     BIGINT UNSIGNED NULL,  -- NULL = global fallback rule
    target_field   ENUM('title', 'description') NOT NULL DEFAULT 'description',
    pattern        VARCHAR(512) NOT NULL,
    capture_group  INT NOT NULL DEFAULT 1,
    priority       INT NOT NULL DEFAULT 0,  -- higher runs first
    CONSTRAINT fk_rules_podcast FOREIGN KEY (podcast_id) REFERENCES podcasts(id) ON DELETE CASCADE,
    INDEX idx_rules_podcast_priority (podcast_id, priority)
);
