use media_source::media_source_item::MediaSourceItem;

pub struct DatabaseUpdater {
    db: sea_orm::DatabaseConnection,
    rx: tokio::sync::mpsc::Receiver<MediaSourceItem>,
}

impl DatabaseUpdater {
    pub fn new(db: sea_orm::DatabaseConnection, rx: tokio::sync::mpsc::Receiver<MediaSourceItem>) -> Self {
        Self {
            db,
            rx
        }
    }

    pub async fn update_database(
        &mut self
    ) -> anyhow::Result<()> {
        while let Some(item) = self.rx.recv().await {
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

}