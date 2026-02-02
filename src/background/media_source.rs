use media_source::media_source_item::MediaSourceItem;
use sea_orm::prelude::async_trait;


#[derive(Debug)]
pub enum MediaSourceEvent {
    FilterResults(Vec<MediaSourceItem>),
    FindResult(Option<MediaSourceItem>),
}

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

/*
#[derive(Debug)]
pub enum MediaSourceCommand {
    Filter {
        query: String,
        reply: oneshot::Sender<MediaSourceEvent>,
    },
    Find {
        id: String,
        reply: oneshot::Sender<MediaSourceEvent>,
    },
}
*/

#[async_trait::async_trait]
pub trait MediaSource: Send + Sync {
    fn id(&self) -> String;
    async fn filter(&self, query: &str) -> Vec<MediaSourceItem>;
    async fn find(&self, id: &str) -> Option<MediaSourceItem>;
}
