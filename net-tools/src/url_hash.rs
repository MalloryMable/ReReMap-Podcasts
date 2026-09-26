use sha2::{Digest, Sha256};

pub fn normalize_feed_url(url: &str) -> String {
    // TODO: strip known tracker params, lowercase host, trim trailing slash
    url.trim().to_string()
}

pub fn hash_url(normalized_url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(normalized_url.as_bytes());
    format!("{:x}", hasher.finalize())
}
