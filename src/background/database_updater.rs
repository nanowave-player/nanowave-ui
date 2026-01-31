use chrono::Utc;
use sea_orm::DatabaseConnection;
use crate::background::database_upsert_item::DatabaseUpsertItem;
use crate::background::metadata_retriever::MetadataRetriever;

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

    pub async fn retrieve_metadata(
        &mut self
    ) -> anyhow::Result<()> {
        while let Some(upsert_item) = self.rx.recv().await {
            
        }
        
        Ok(())
    }

/*
    async fn upsert_item(&self, id: i32, file_id: String, media_type: item::MediaType, location: String, meta: &MediaSourceMetadata) -> ActiveModelEx {
        // todo: improve this
        // see https://www.sea-ql.org/blog/2025-11-25-sea-orm-2.0/
        let db = self.db.clone();
        let now = Utc::now();
        let cover = meta.cover.clone();

        let cover_hash = if cover.is_some() {
            cover.unwrap().hash
        } else {
            String::from("")
        };



        // let file_id_item = self.find_file_id()


        // if id == 0 insert, otherwise update
        let builder = if id == 0 {
            ActiveModel::builder()
                .set_file_id(file_id)
                .set_media_type(media_type)
                .set_location(location.trim_start_matches('/'))
                .set_cover_hash(cover_hash)
                .set_last_scan_random_key("")
                .set_date_modified(now)
            //.add_metadatum(metadata_items)

        } else {
            ActiveModel::builder()
                .set_id(id)
                .set_file_id(file_id)
                .set_media_type(media_type)
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
        self.add_metadata(&mut result.metadata, Genre, meta.genre.clone(), now);
        self.add_metadata(&mut result.metadata, Artist, meta.artist.clone(), now);
        self.add_metadata(&mut result.metadata, Title, meta.title.clone(), now);
        self.add_metadata(&mut result.metadata, Album, meta.album.clone(), now);
        self.add_metadata(&mut result.metadata, Composer, meta.composer.clone(), now);
        self.add_metadata(&mut result.metadata, Series, meta.series.clone(), now);
        self.add_metadata(&mut result.metadata, Part, meta.part.clone(), now);

        if !meta.chapters.is_empty() {
            let chapters_json_result = serde_json::to_string(&meta.chapters);
            if let Ok(chapters_json) = chapters_json_result {
                let chapters_model = items_json_metadata::ActiveModel::builder()
                    .set_tag_field(Chapters)
                    .set_value(chapters_json)
                    .set_date_modified(now);
                result.json.push(chapters_model);
            }

        }

        let res = result.save(&db).await;

        res.unwrap()
    }

    fn add_metadata(&self, metadata: &mut HasManyModel<items_metadata::Entity>, tag_field: items_metadata::TagField, value: Option<String>, date_modified: DateTime<Utc>) {
        if value.is_some() {
            metadata.push(items_metadata::ActiveModel::builder()
                .set_tag_field(tag_field)
                .set_value(value.unwrap())
                .set_date_modified(date_modified));
        }
    }
*/
}