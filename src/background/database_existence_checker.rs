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

            println!("db_existance_checker receiving: {:?}", file);

            let file_id_result = file_id::get_file_id(file.as_path());
            if file_id_result.is_err() {
                // todo: Logging
                println!("db_existance_checker err: {:?}", file_id_result);

                continue;
            }
            let file_id = file_id_result?;
            let existing_record_result = self.load_existing_record(&file_id).await;
            if existing_record_result.is_err() {
                // todo: Logging
                println!("db_existance_checker err: {:?}", existing_record_result);
                continue;
            }


            let existing_record_option = existing_record_result.unwrap();
            if !self.needs_upsert(&file, &existing_record_option) {
                println!("db_existance_checker no upsert needed: {:?}", file);
                continue;
            }


            let upsert_item = DatabaseUpsertItem {
                file,
                file_id,
                media_source_item: None,
                model: existing_record_option,
            };
            println!("db_existance_checker update required");

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
        println!("db_existance_checker exiting");
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
        true
    }

}