use chrono::{Timelike, Utc};
use std::time::Duration;
use chrono::NaiveTime;
use crate::background::media_source::{MediaSource, MediaSourceCommand, MediaSourceHistoryItem};
use crate::config::Config;
use crate::entity::items_json_metadata::JsonTagField;
use crate::entity::items_metadata::TagField;
use crate::entity::{item, items_json_metadata, items_metadata, items_progress_history};
use media_source::media_source_chapter::MediaSourceChapter;
use media_source::media_source_image_codec::MediaSourceImageCodec;
use media_source::media_source_item::MediaSourceItem;
use media_source::media_source_metadata::MediaSourceMetadata;
use media_source::media_source_picture::MediaSourcePicture;
use sea_orm::compound::HasMany;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::sea_query::prelude::serde_json;
use sea_orm::{ColumnTrait, QueryOrder};
use sea_orm::DatabaseConnection;
use sea_orm::QueryFilter;
use tokio::sync::mpsc::UnboundedReceiver;
use crate::background::file_media_source_playback_history::FileMediaSourcePlaybackHistory;
use crate::entity;
use crate::entity::item::{ActiveModelEx, ModelEx};

#[derive(Clone)]
pub struct FileMediaSource {
    pub config: Config,
    pub db: DatabaseConnection,
    playback_history: FileMediaSourcePlaybackHistory,
}

impl FileMediaSource {

    pub fn new(config: Config, db: DatabaseConnection) -> Self{
        Self {
            config,
            db: db.clone(),
            playback_history: FileMediaSourcePlaybackHistory::new(db.clone())
        }
    }
    pub async fn run(
        mut self,
        mut cmd_rx: UnboundedReceiver<MediaSourceCommand>,
    ) {
        while let Some(cmd) = cmd_rx.recv().await {
            match cmd {
                MediaSourceCommand::Filter(cmd) => {
                    let results: Vec<MediaSourceItem> = self.filter(&cmd.query).await;
                    (cmd.callback)(results);
                }

                MediaSourceCommand::Find(cmd) => {
                    let result: Option<MediaSourceItem> = self.find(&cmd.id).await;
                    (cmd.callback)(result);
                }
            }
        }
    }

    async fn find_item(&self, id: &str) -> Option<ModelEx> {
        let db = self.db.clone();
        let item = item::Entity::load()
            .filter(item::Column::Id.eq(id))
            .with(items_metadata::Entity)
            .with(items_json_metadata::Entity)
            .one(&db)
            .await;

        if item.is_err() {
            return None;
        }
        item.unwrap()
    }

    pub fn build_location_full_path(&self, location:String) -> String {
        // format!("{}/{}", self.base_path.clone().trim_end_matches('/'), i.location.trim_start_matches('/').to_string()),
        format!("{}/{}", self.config.base_path.trim_end_matches('/'), location.trim_start_matches('/').to_string())
    }
    pub fn map_db_model_to_media_item(&self, i: &item::ModelEx, metadata_option: Option<&HasMany<items_metadata::Entity>>, json_option: Option<&HasMany<items_json_metadata::Entity>>) -> MediaSourceItem {
        let cache_path = self.config.cache_path.clone();
        // let mut base_path = self.config.base_path.clone();
        let mut title : String = String::from("");
        let mut genre : Option<String> = None;
        let mut artist : Option<String> = None;
        let mut album : Option<String> = None;
        let mut composer : Option<String> = None;
        let mut series : Option<String> = None;
        let mut part : Option<String> = None;
        let cover = Some(MediaSourcePicture {
            cache_dir: cache_path,
            hash: i.cover_hash.clone(),
            codec: MediaSourceImageCodec::Jpeg
        });
        let filename = i.location.split('/').last();
        if filename.is_some() {
            let filename_no_ext = filename.unwrap().split('.').next();
            if filename_no_ext.is_some() {
                title = filename_no_ext.unwrap().to_string();
            }
        }

        if let Some(metadata) = metadata_option {
            for tag in metadata {
                match tag.tag_field {
                    TagField::Title => title = tag.value.clone(),
                    TagField::Genre => genre = Some(tag.value.clone()),
                    TagField::Artist => artist = Some(tag.value.clone()),
                    TagField::Album => album = Some(tag.value.clone()),
                    TagField::Composer => composer = Some(tag.value.clone()),
                    TagField::Series => series = Some(tag.value.clone()),
                    TagField::Part => part = Some(tag.value.clone()),
                };
            }
        }

        let mut chapters: Vec<MediaSourceChapter> = Vec::new();

        if let Some(json) = json_option {
            for json_tag in json {
                match json_tag.tag_field {
                    JsonTagField::Chapters => {
                        if let Ok(chaps) = serde_json::from_str(&json_tag.value) {
                            chapters = chaps;
                        }
                    },
                    _ => {}
                }
            }
        }


        
        MediaSourceItem {
            id: i.id.to_string(),
            location: self.build_location_full_path(i.location.clone()),
            title: title.clone(),
            media_type: media_source::media_type::MediaType::Unspecified,
            metadata: MediaSourceMetadata {
                title: Some(title.clone()),
                artist,
                album,
                genre,
                composer,
                series,
                part,
                cover,
                chapters
            },
        }
    }
}


#[async_trait]
impl MediaSource for FileMediaSource {
    async fn filter(&self, query: &str) -> Vec<MediaSourceItem> {
        let db = self.db.clone();

        // let q = query.to_lowercase();
        let media_type = match query {
            "4" => item::MediaType::Music,
            "2" => item::MediaType::Audiobook,
            _ => item::MediaType::Unspecified
        };

        let items = item::Entity::load()
            .filter(item::Column::MediaType.eq(media_type))
            .with(items_metadata::Entity)
            .all(&db)
            .await;
        if items.is_err() {
            return vec![MediaSourceItem{
                id: "".to_string(),
                location: "".to_string(),
                title: "error".to_string(),
                media_type: media_source::media_type::MediaType::Unspecified,
                metadata: MediaSourceMetadata {
                    artist: None,
                    title: None,
                    album: None,
                    genre: None,
                    composer: None,
                    series: None,
                    part: None,
                    cover: None,
                    chapters: vec![],
                },
            }];
        }

        let items = items.unwrap();
        let result: Vec<MediaSourceItem> = items.iter().map(|i| {
            self.map_db_model_to_media_item(i, Some(&i.metadata), Some(&i.json))
        }).collect();

        result
    }

    async fn find(&self, id: &str) -> Option<MediaSourceItem> {
        let item = self.find_item(id).await;
        if let Some(i) = item {
            return Some(self.map_db_model_to_media_item(&i, Some(&i.metadata), Some(&i.json)));
        }
        None
    }



    async fn history_filter(&self, query: &str) -> Vec<MediaSourceHistoryItem> {

        let history_items = self.playback_history.filter(query).await;
        let mut filtered_items: Vec<MediaSourceHistoryItem> = vec![];
        for history_item in history_items {
            let model = history_item.item.unwrap();
            let item = MediaSourceHistoryItem {
                item: self.map_db_model_to_media_item(&model, Some(&model.metadata), Some(&model.json)),
                session_key: history_item.session_key,
                position: naive_time_to_duration(history_item.position),
                date_modified: history_item.date_modified.into(),
            };
            filtered_items.push(item);
        }
        filtered_items

        /*
        let db = self.db.clone();
        let result = entity::items_progress_history::Entity::load()
            .with(item::Entity)
            .order_by_desc(item::Column::DateModified)
            .all(&db)
            .await;

        let mut filtered_items: Vec<MediaSourceHistoryItem> = vec![];
        if let Ok(history_items) = result {
            for history_item in history_items {
                let model = history_item.item.unwrap();
                let item = MediaSourceHistoryItem {
                    item: self.map_db_model_to_media_item(&model, Some(&model.metadata), Some(&model.json)),
                    session_key: history_item.session_key,
                    position: naive_time_to_duration(history_item.position),
                    date_modified: history_item.date_modified.into(),
                };
                filtered_items.push(item);
            }
        }
        filtered_items

         */
    }

    async fn history_update(&self, media_item_id: &str, random_session_key: &str, new_position: Duration) -> bool {
        let item_option = self.find_item(&media_item_id).await;
        if let Some(item) = item_option {
            return self.playback_history.update(ActiveModelEx::from(item), random_session_key, new_position).await;
        }
        false
        /*
        let db = self.db.clone();

        let item_option = self.find_item(&media_item_id).await;
        let now = Utc::now();

        if let Some(item) = item_option {
            let builder = items_progress_history::ActiveModel::builder()
                .set_session_key(random_session_key)
                .set_item(item)
                .set_position(duration_to_naive_time(new_position))
                .set_date_modified(now);

            let result = builder
                .save(&db)
                .await;

            if result.is_err() {
                return false;
            }
        }
        true

         */
    }
}


fn naive_time_to_duration(naive_time: NaiveTime) -> Duration {
    let seconds_since_midnight = naive_time.num_seconds_from_midnight();
    let nanoseconds = naive_time.nanosecond();
    Duration::new(seconds_since_midnight.into(), nanoseconds)
}
