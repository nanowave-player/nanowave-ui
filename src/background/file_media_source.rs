use crate::background::file_media_source_playback_history::FileMediaSourcePlaybackHistory;
use crate::background::media_source::{MediaSource, MediaSourceCommand};
use crate::config::Config;
use crate::entity::item::ModelEx;
use crate::entity::items_json_metadata::JsonTagField;
use crate::entity::items_metadata::TagField;
use crate::entity::{item, items_json_metadata, items_metadata, items_progress_history};
use chrono::NaiveTime;
use chrono::Timelike;
use media_source::media_source_chapter::MediaSourceChapter;
use media_source::media_source_image_codec::MediaSourceImageCodec;
use media_source::media_source_item::MediaSourceItem;
use media_source::media_source_metadata::MediaSourceMetadata;
use media_source::media_source_picture::MediaSourcePicture;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::sea_query::prelude::serde_json;
use sea_orm::DatabaseConnection;
use sea_orm::QueryFilter;
use sea_orm::ColumnTrait;
use std::time::Duration;
use media_source::media_source_history_item::MediaSourceHistoryItem;
use media_source::media_source_session_key::MediaSourceSessionKey;
use tokio::sync::mpsc::UnboundedReceiver;

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


                    // todo: what about progress
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
    pub fn map_db_model_to_media_item(&self, i: &ModelEx, position: Option<Duration>) -> MediaSourceItem {



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

        // , metadata_option: Option<&HasMany<items_metadata::Entity>>, json_option: Option<&HasMany<items_json_metadata::Entity>>
        let metadata_option = Some(&i.metadata);

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

        if let Some(json) = Some(&i.json) {
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
            position,
            history: vec![]
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
                position: None,
                history: vec![]
            }];
        }

        let items = items.unwrap();
        let result: Vec<MediaSourceItem> = items.iter().map(|i| {
            self.map_db_model_to_media_item(i, None)
        }).collect();

        result
    }

    async fn find(&self, id: &str) -> Option<MediaSourceItem> {
        let item = self.find_item(id).await;

        let position = {
            let history_item_option = self.playback_history.find_latest(id).await;
            if let Some(history_item) = history_item_option {
                Some(naive_time_to_duration(history_item.position))
            } else {
                None
            }
        };


        if let Some(i) = item {
            return Some(self.map_db_model_to_media_item(&i, position));
        }
        None
    }

    async fn history_latest(&self) -> Option<MediaSourceHistoryItem> {
        let latest_option = self.playback_history.find_latest("").await;
        if let Some(model) = latest_option &&
            let Ok(session_key) = MediaSourceSessionKey::parse_string(model.session_key.as_str()) {
            let item_model = model.item.unwrap();
            let media_source_item = self.map_db_model_to_media_item(&item_model, Some(naive_time_to_duration(model.position)));
            let hist_item = MediaSourceHistoryItem::new(media_source_item, session_key, naive_time_to_duration(model.position), model.date_modified.into());
            return Some(hist_item);
        }
        None
    }

    async fn history_filter(&self, query: &str) -> Vec<MediaSourceHistoryItem> {
        let mut filtered_items: Vec<MediaSourceHistoryItem> = vec![];

        let history_items: Vec<items_progress_history::ModelEx> = self.playback_history.filter(query).await;

        for history_item in history_items {

            // HasOne relationship
            let model = history_item.item.unwrap();

            if let Ok(session_key) = MediaSourceSessionKey::parse_string(history_item.session_key.as_str())  {
                let position = naive_time_to_duration(history_item.position);
                let item = MediaSourceHistoryItem {
                    item: self.map_db_model_to_media_item(&model, Some(position.clone())),
                    session_key,
                    position: position.clone(),
                    date_modified: history_item.date_modified.into(),
                };
                filtered_items.push(item);
            }
        }

        filtered_items

    }

    async fn history_update(&self, history_item: MediaSourceHistoryItem) -> bool {
        self.playback_history.update(history_item).await
    }
}


fn naive_time_to_duration(naive_time: NaiveTime) -> Duration {
    let seconds_since_midnight = naive_time.num_seconds_from_midnight();
    let nanoseconds = naive_time.nanosecond();
    Duration::new(seconds_since_midnight.into(), nanoseconds)
}
