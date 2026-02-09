use media_source::media_source_item::MediaSourceItem;
use sea_orm::prelude::async_trait;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use rand::distributions::{Alphanumeric, DistString};

pub struct FilterCommand {
    pub query: String,
    pub callback: Box<dyn FnOnce(Vec<MediaSourceItem>) + Send>,
}

pub struct FindCommand {
    pub id: String,
    pub callback: Box<dyn FnOnce(Option<MediaSourceItem>) + Send>,
}

pub enum MediaSourceCommand {
    Filter(FilterCommand),
    Find(FindCommand),
}

const SESSION_KEY_MAX_AGE: u128 = 1000 * 60 * 30; // 30 mins

#[derive(Debug, Clone, PartialEq)]
pub struct SessionKey {
    pub time: u128,
    pub key: String,
    pub expires: u128,
}

impl SessionKey {

    pub fn new() -> Self {
        Self::create(Self::now(), Self::key())
    }

    fn create(time: u128, key: String) -> Self {
        Self {
            time,
            key,
            expires: time + SESSION_KEY_MAX_AGE,
        }
    }

    pub fn now() -> u128 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
    }

    pub fn key() -> String {
        Alphanumeric.sample_string(&mut rand::thread_rng(), 16)
    }

    pub fn extend_validity(&mut self) {
        self.expires = Self::now() + SESSION_KEY_MAX_AGE;
    }

    /*
    pub fn renew(&mut self) {
        self.time = Self::now();
        self.key = Self::key();
        self.expires = self.time + SESSION_MAX_AGE;
    }
*/

    pub fn is_expired(&self) -> bool {
        Self::now() > self.expires
    }

    pub fn to_string(&self) -> String {
        format!("{}.{}", self.time, self.key)
    }


    pub fn parse_string(s: &str) -> Result<Self, &'static str> {
        if s.is_empty() {
            return Err("Empty string");
        }

        let parts: Vec<&str> = s.splitn(2, '.').collect();
        if parts.len() != 2 {
            return Err("No dot separator found");
        }

        let time_part = parts[0];
        let key_part = parts[1];

        let time = time_part.parse::<u128>()
            .map_err(|_| "Invalid u128")?;

        Ok(Self {
            time,
            key: key_part.to_string(),
            expires: time + SESSION_KEY_MAX_AGE,
        })
    }
}


#[derive(Debug, Clone)]
pub struct MediaSourceHistoryItem {
    pub item: MediaSourceItem,
    pub session_key: SessionKey,
    pub position: Duration,
    pub date_modified: SystemTime
}

impl MediaSourceHistoryItem {
    pub fn new(item: MediaSourceItem, session_key: SessionKey, position: Duration, date_modified: SystemTime) -> Self {
        Self {
            item,
            session_key,
            position,
            date_modified
        }
    }
}

#[async_trait::async_trait]
pub trait MediaSource: Send + Sync {
    async fn filter(&self, query: &str) -> Vec<MediaSourceItem>;
    async fn find(&self, media_item_id: &str) -> Option<MediaSourceItem>;
    async fn history_latest(&self) -> Option<MediaSourceHistoryItem>;
    async fn history_filter(&self, query: &str) -> Vec<MediaSourceHistoryItem>;

    async fn history_update(&self, history_item: MediaSourceHistoryItem) -> bool;
}
