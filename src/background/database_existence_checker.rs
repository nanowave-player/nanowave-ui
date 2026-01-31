use sea_orm::{ColumnTrait, DbErr};
use sea_orm::QueryFilter;
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use file_id::FileId;
use sea_orm::EntityTrait;
use crate::background::database_upsert_item::DatabaseUpsertItem;
use crate::entity::item;
use crate::entity::item::Model;
#[derive(Debug, Clone)]
pub enum DatabaseExistenceCheckerError {
    FileId,
    Database(DbErr),

}


pub struct DatabaseExistenceChecker {
    db: sea_orm::DatabaseConnection,
    rx: tokio::sync::mpsc::Receiver<PathBuf>,
    tx: tokio::sync::mpsc::Sender<DatabaseUpsertItem>,
}

impl DatabaseExistenceChecker {
    pub fn new(
        db: sea_orm::DatabaseConnection,
        rx: tokio::sync::mpsc::Receiver<PathBuf>,
        tx: tokio::sync::mpsc::Sender<DatabaseUpsertItem>,
    ) -> Self {
        Self {
            db,
            rx,
            tx
        }
    }

    pub async fn check_items_for_needed_update(
        &mut self
    ) -> anyhow::Result<()> {
        while let Some(file) = self.rx.recv().await {

            let file_id_result = file_id::get_file_id(file.as_path());
            if file_id_result.is_err() {
                // todo: Logging
                continue;
            }
            let file_id = file_id_result?;
            let existing_record_result = self.load_existing_record(&file_id).await;
            if existing_record_result.is_err() {
                // todo: Logging
                continue;
            }


            let existing_record_option = existing_record_result.unwrap();
            if !self.needs_upsert(&file, &existing_record_option) {
                continue;
            }


            let upsert_item = DatabaseUpsertItem {
                file,
                file_id,
                media_source_item: None,
                model: existing_record_option,
            };
            self.tx.send(upsert_item).await?;


            // self.upsert_item(&file, &existing_record_option);


            /*
            media::ActiveModel {
                title: Set(item.title),
                album: Set(item.album),
                artist: Set(item.artist),
                composer: Set(item.composer),
                duration: Set(item.duration),
                path: Set(item.path.to_string_lossy().to_string()),
                ..Default::default()
            }
                .insert(&db)
                .await?;

             */
        }
        Ok(())
    }

    pub async fn load_existing_record(&self, file_id: &FileId) -> Result<Option<Model>, DatabaseExistenceCheckerError> {


        let file_id_str = format!("{:?}", file_id);

        let item_result = item::Entity::find()
            .filter(item::Column::FileId.eq(file_id_str.clone()))
            .one(&self.db)
            .await;

        if item_result.is_err() {
            return Err(DatabaseExistenceCheckerError::Database(item_result.unwrap_err()));
        }

        Ok(item_result.unwrap())
    }

    fn needs_upsert(&self, file: &PathBuf, record_option: &Option<Model>) -> bool {
        if let Ok(file_meta) = file.as_path().metadata() &&
            let Some(record) = record_option &&
            let Ok(file_modified) = file_meta.modified() {
            let file_modified_chrono : DateTime<Utc> = file_modified.into();
            return record.date_modified < file_modified_chrono;
        }
        false
    }

    fn upsert_item(&self, path: &PathBuf, existing_model: &Option<Model>) {

        /*
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
        */
    }
}