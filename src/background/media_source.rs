use media_source::media_source_item::MediaSourceItem;
use sea_orm::prelude::async_trait;
use std::time::{Duration, SystemTime};

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

pub struct MediaSourceHistoryItem {
    pub item: MediaSourceItem,
    pub session_key: String,
    pub position: Duration,
    pub date_modified: SystemTime
}

#[async_trait::async_trait]
pub trait MediaSource: Send + Sync {
    async fn filter(&self, query: &str) -> Vec<MediaSourceItem>;
    async fn find(&self, media_item_id: &str) -> Option<MediaSourceItem>;
    async fn history_filter(&self, query: &str) -> Vec<MediaSourceHistoryItem>;
    
    async fn history_update(&self, media_item_id: &str, random_session_key: &str, new_position: Duration) -> bool;
}
