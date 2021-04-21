use serde::{Deserialize, Serialize};
use lru_time_cache::LruCache;
use std::time::Duration;
use launcher_extension_api::launcher::config::Configurable;

#[derive(Deserialize, Serialize)]
pub struct Config {
    rate_limit: u64,
    ip_limit: Option<usize>,
    max_try: u64,
}

impl Configurable for Config {}

impl Default for Config {
    fn default() -> Self {
        Config {
            rate_limit: 10,
            ip_limit: Some(1000),
            max_try: 3
        }
    }
}


impl Config {
    pub fn get_blocker(self) -> Blocker {
        let duration = Duration::from_secs(self.rate_limit);
        let cache = match self.ip_limit {
            Some(size) => {
                LruCache::with_expiry_duration_and_capacity(duration, size)
            }
            None => {
                LruCache::with_expiry_duration(duration)
            }
        };
        Blocker {
            cache,
            max_try: self.max_try,
        }
    }
}

pub struct Blocker {
    cache: LruCache<String, u64>,
    max_try: u64,
}

impl Blocker {
    pub fn limit(&mut self, ip: &str) -> bool {
        let mut count = *self.cache.get(ip).unwrap_or(&0);
        if count >= self.max_try {
            true
        } else {
            count += 1;
            self.cache.insert(ip.to_string(), count);
            false
        }
    }
}

