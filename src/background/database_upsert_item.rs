use crate::entity::item::Model;
use file_id::FileId;
use media_source::media_source_item::MediaSourceItem;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DatabaseUpsertItem {
    pub file: PathBuf,
    pub file_id: FileId,
    pub media_source_item: Option<MediaSourceItem>,
    pub model: Option<Model>
}

