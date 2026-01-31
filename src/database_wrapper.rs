use crate::entity::{item, items_json_metadata, items_metadata, items_progress_history};
use sea_orm::{Database, DatabaseConnection, DatabaseConnectionType};
use std::path::Path;
use sea_orm_migration::MigratorTrait;
use crate::migrator::Migrator;

pub struct DatabaseWrapper {
    db_url: String,
    db_exists: bool
}

impl DatabaseWrapper {
    pub fn new(base_dir: String) -> Self {
        let db_path = format!(
            "{}/{}",
            base_dir.clone().trim_end_matches("/"),
            String::from("nanowave.db")
        );
        let db_exists = Path::new(&db_path).exists();
        let db_url = format!("sqlite://{}?mode=rwc", db_path);
        Self {
            db_url,
            db_exists,
        }
    }

    pub async fn connect(&mut self) -> Result<DatabaseConnection, sea_orm::error::DbErr> {
        let db = Database::connect(self.db_url.clone()).await?;
        // todo: dirty hack to prevent startup failure if db exists
        // this has to be solved with migrations or at least better than this
        if !self.db_exists {
            db.get_schema_builder()
                .register(item::Entity)
                .register(items_metadata::Entity)
                .register(items_json_metadata::Entity)
                .register(items_progress_history::Entity)
                .apply(&db)
                .await?;
        }
        Migrator::up(&db, None).await?;
        Ok(db)
    }
}