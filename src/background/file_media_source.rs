use crate::background::media_source::{MediaSource, MediaSourceCommand};
use crate::config::Config;
use crate::entity::items_json_metadata::JsonTagField;
use crate::entity::items_metadata::TagField;
use crate::entity::{item, items_json_metadata, items_metadata};
use media_source::media_source_chapter::MediaSourceChapter;
use media_source::media_source_image_codec::MediaSourceImageCodec;
use media_source::media_source_item::MediaSourceItem;
use media_source::media_source_metadata::MediaSourceMetadata;
use media_source::media_source_picture::MediaSourcePicture;
use sea_orm::compound::HasMany;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::sea_query::prelude::serde_json;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::QueryFilter;
use tokio::sync::mpsc::UnboundedReceiver;


#[derive(Clone)]
pub struct FileMediaSource {
    pub config: Config,
    pub db: DatabaseConnection,
}

impl FileMediaSource {

    pub fn new(config: Config, db: DatabaseConnection) -> Self{
        Self {
            config,
            db
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

    pub fn build_location_full_path(&self, location:String) -> String {
        // format!("{}/{}", self.base_path.clone().trim_end_matches('/'), i.location.trim_start_matches('/').to_string()),
        format!("{}/{}", self.config.base_path.trim_end_matches('/'), location.trim_start_matches('/').to_string())
    }
    pub fn map_db_model_to_media_item(&self, i: &item::ModelEx, metadata: &HasMany<items_metadata::Entity>, json: &HasMany<items_json_metadata::Entity>) -> MediaSourceItem {
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

        let mut chapters: Vec<MediaSourceChapter> = Vec::new();

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
            self.map_db_model_to_media_item(i, &i.metadata, &i.json)
        }).collect();

        result
    }

    async fn find(&self, id: &str) -> Option<MediaSourceItem> {
        let db = self.db.clone();
        let items = item::Entity::load()
            .filter(item::Column::Id.eq(id))
            .with(items_metadata::Entity)
            .with(items_json_metadata::Entity)
            .one(&db)
            .await;

        if items.is_err() {
            return None;
        }

        let items = items.unwrap();

        if let Some(i) = items {
            return Some(self.map_db_model_to_media_item(&i, &i.metadata, &i.json));
        }
        None
    }
}