use crate::background::database_upsert_item::DatabaseUpsertItem;
use crate::entity::item::MediaType;
use crate::entity::{item, items_json_metadata, items_metadata};
use chrono::{DateTime, Utc};
use media_source::media_source_metadata::MediaSourceMetadata;
use media_source::media_type;
use sea_orm::sea_query::prelude::serde_json;
use sea_orm::{DatabaseConnection, HasManyModel};

pub struct DatabaseUpdater {
    db: DatabaseConnection,
    rx: tokio::sync::mpsc::Receiver<DatabaseUpsertItem>,

}

impl DatabaseUpdater {
    pub fn new(
        db: DatabaseConnection,
        rx: tokio::sync::mpsc::Receiver<DatabaseUpsertItem>,
        ) -> DatabaseUpdater {
        Self {
            db,
            rx,
        }
    }

    pub async fn update_items(
        &mut self
    ) -> anyhow::Result<()> {
        while let Some(upsert_item) = self.rx.recv().await {
            self.upsert_item(upsert_item).await;
        }
        
        Ok(())
    }
    async fn upsert_item(&self, upsert_item: DatabaseUpsertItem) {
        let db = self.db.clone();
        let now = Utc::now();



        let (meta, location, media_type) = if let Some(media_source_item) = upsert_item.clone().media_source_item {
            (media_source_item.metadata.clone(), media_source_item.location, media_source_item.media_type)
        } else {
            (MediaSourceMetadata::empty(), String::from(""), media_type::MediaType::Unspecified)
        };

        /*
        let id = if let Some(upsert_item_model) = upsert_item.clone().media_source_item {
            upsert_item_model.id.clone()
        } else {
            String::from("")
        };
        */
        let id = if let Some(upsert_item_model) = upsert_item.clone().model {
            upsert_item_model.id
        } else {
            0
        };

        let cover = meta.cover.clone();


        // let id = upsert_item.clone().model.unwrap().id;
        let file_id = upsert_item.file_id;
        let file_id_string = format!("{:?}", file_id);
        let cover_hash = if cover.is_some() {
            cover.unwrap().hash
        } else {
            String::from("")
        };


        let db_media_type = map_media_type(media_type);

        let builder = if id == 0 {
            item::ActiveModel::builder()
                .set_file_id(file_id_string)
                .set_media_type(db_media_type)
                .set_location(location.trim_start_matches('/'))
                .set_cover_hash(cover_hash)
                .set_last_scan_random_key("")
                .set_date_modified(now)
            //.add_metadatum(metadata_items)

        } else {
            item::ActiveModel::builder()
                .set_id(id)
                .set_file_id(file_id_string)
                .set_media_type(db_media_type)
                .set_location(location.trim_start_matches('/'))
                .set_cover_hash(cover_hash)
                .set_last_scan_random_key("")
                .set_date_modified(now)

        };


        let mut result = builder
            // .add_metadatum()
            // .add_picture()
            // .add_progress_history()
            .save(&db)
            .await
            .expect("todo");

        // now sync the metadata
        // todo: handle multi persons with comma separated values
        self.add_metadata(&mut result.metadata, items_metadata::TagField::Genre, meta.genre.clone(), now);
        self.add_metadata(&mut result.metadata, items_metadata::TagField::Artist, meta.artist.clone(), now);
        self.add_metadata(&mut result.metadata, items_metadata::TagField::Title, meta.title.clone(), now);
        self.add_metadata(&mut result.metadata, items_metadata::TagField::Album, meta.album.clone(), now);
        self.add_metadata(&mut result.metadata, items_metadata::TagField::Composer, meta.composer.clone(), now);
        self.add_metadata(&mut result.metadata, items_metadata::TagField::Series, meta.series.clone(), now);
        self.add_metadata(&mut result.metadata, items_metadata::TagField::Part, meta.part.clone(), now);

        if !meta.chapters.is_empty() {
            let chapters_json_result = serde_json::to_string(&meta.chapters);
            if let Ok(chapters_json) = chapters_json_result {
                let chapters_model = items_json_metadata::ActiveModel::builder()
                    .set_tag_field(items_json_metadata::JsonTagField::Chapters)
                    .set_value(chapters_json)
                    .set_date_modified(now);
                result.json.push(chapters_model);
            }

        }

        let _ = result.save(&db).await;

        // res.unwrap()

    }

    fn add_metadata(&self, metadata: &mut HasManyModel<items_metadata::Entity>, tag_field: items_metadata::TagField, value: Option<String>, date_modified: DateTime<Utc>) {
        if value.is_some() {
            metadata.push(items_metadata::ActiveModel::builder()
                .set_tag_field(tag_field)
                .set_value(value.unwrap())
                .set_date_modified(date_modified));
        }
    }


}

fn map_media_type(media_type: media_type::MediaType) -> MediaType {
    match media_type {
        media_type::MediaType::Audiobook => MediaType::Audiobook,
        media_type::MediaType::Music => MediaType::Music,
        _ => MediaType::Unspecified,
        // media_type::MediaType::Unspecified => MediaType::Unspecified,
    }
}