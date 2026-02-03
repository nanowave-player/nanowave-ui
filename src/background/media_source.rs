use media_source::media_source_item::MediaSourceItem;
use sea_orm::prelude::async_trait;

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

#[async_trait::async_trait]
pub trait MediaSource: Send + Sync {
    async fn filter(&self, query: &str) -> Vec<MediaSourceItem>;
    async fn find(&self, id: &str) -> Option<MediaSourceItem>;
}
