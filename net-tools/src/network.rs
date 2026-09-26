use reqwest::{Client, StatusCode, header};
use rss::Channel;
use chrono::{DateTime, NaiveDateTime};
use std::time::Duration;

//TODO: make sure I can remove audio_url as well

/// The output payload sent back to the caller.
#[derive(Debug)]
pub enum FetchResult {
    NotModified,             // 304 response, skip processing
    Success(ParsedFeed),     // 200 response, new data
    Error(String),
}

#[derive(Debug)]
pub struct ParsedFeed {
    pub title: String,
    /// `response.status()` is never a redirect code by the time we see it
    pub final_url: String,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub episodes: Vec<ParsedEpisode>,
}

#[derive(Debug)]
pub struct ParsedEpisode {
    pub guid: String,
    pub title: String,
    pub audio_url: String,
    pub duration_sec: Option<u32>, // Transient value for filtering ads
    pub published_at: Option<NaiveDateTime>,
    pub raw_description: Option<String>,
}

pub struct NetworkModel {
    client: Client,
}

impl NetworkModel {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("ReReMapper/0.1 (Linux)")
            .timeout(Duration::from_secs(15))
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
            .expect("Failed to build HTTP client");

        Self { client }
    }

    /// Fetches and parses a feed. Takes etag/last_modified from the DB for conditional GETs.
    pub async fn fetch_feed(
        &self,
        url: &str,
        etag: Option<&str>,
        last_modified: Option<&str>,
    ) -> FetchResult {
        let mut request = self.client.get(url);

        if let Some(e) = etag {
            request = request.header(header::IF_NONE_MATCH, e);
        }
        if let Some(lm) = last_modified {
            request = request.header(header::IF_MODIFIED_SINCE, lm);
        }

        let response = match request.send().await {
            Ok(res) => res,
            Err(e) => return FetchResult::Error(e.to_string()),
        };

        let status = response.status();

        if status == StatusCode::NOT_MODIFIED {
            return FetchResult::NotModified;
        }
        if !status.is_success() {
            return FetchResult::Error(format!("HTTP Error: {}", status));
        }

        let final_url = response.url().to_string();
        let new_etag = response.headers().get(header::ETAG)
            .and_then(|v| v.to_str().ok()).map(String::from);
        let new_last_modified = response.headers().get(header::LAST_MODIFIED)
            .and_then(|v| v.to_str().ok()).map(String::from);

        let body = match response.bytes().await {
            Ok(b) => b,
            Err(e) => return FetchResult::Error(e.to_string()),
        };

        let channel = match Channel::read_from(&body[..]) {
            Ok(c) => c,
            Err(e) => return FetchResult::Error(e.to_string()),
        };

        let mut parsed_episodes = Vec::new();

        for item in channel.items() {
            let audio_url = item.enclosure().map(|e| e.url().to_string()).unwrap_or_default();
            let guid = item.guid().map(|g| g.value().to_string()).unwrap_or_else(|| audio_url.clone());

            if audio_url.is_empty() {
                continue; // no enclosure -> nothing to play, skip
            }

            let duration_sec = item.itunes_ext()
                .and_then(|it| it.duration())
                .and_then(Self::parse_duration_to_seconds);

            let published_at = item.pub_date()
                .and_then(|d| DateTime::parse_from_rfc2822(d).ok())
                .map(|d| d.naive_utc());

            parsed_episodes.push(ParsedEpisode {
                guid,
                title: item.title().unwrap_or("Untitled").to_string(), // TODO: better error
                audio_url,
                duration_sec,
                published_at,
                raw_description: item.description().map(String::from),
            });
        }

        FetchResult::Success(ParsedFeed {
            title: channel.title().to_string(),
            final_url,
            etag: new_etag,
            last_modified: new_last_modified,
            episodes: parsed_episodes,
        })
    }

    fn parse_duration_to_seconds(duration_str: &str) -> Option<u32> {
        let parts: Vec<&str> = duration_str.trim().split(':').collect();
        match parts.len() {
            3 => {
                let h: u32 = parts[0].parse().ok()?;
                let m: u32 = parts[1].parse().ok()?;
                let s: u32 = parts[2].parse().ok()?;
                Some((h * 3600) + (m * 60) + s)
            }
            2 => {
                let m: u32 = parts[0].parse().ok()?;
                let s: u32 = parts[1].parse().ok()?;
                Some((m * 60) + s)
            }
            1 => parts[0].parse().ok(),
            _ => None,
        }
    }
}
