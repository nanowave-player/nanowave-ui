use std::path::PathBuf;
use media_source::media_source_item::MediaSourceItem;
use media_source::media_source_metadata::MediaSourceMetadata;
use media_source::media_type::MediaType;

pub struct MediaAnalyzer {
    rx: tokio::sync::mpsc::Receiver<PathBuf>,
    tx: tokio::sync::mpsc::Sender<MediaSourceItem>,
}

impl MediaAnalyzer {
    pub fn new(rx: tokio::sync::mpsc::Receiver<PathBuf>, tx: tokio::sync::mpsc::Sender<MediaSourceItem>) -> Self {
        Self { rx, tx }
    }
    pub async fn analyze_metadata(&mut self) {
        while let Some(path) = self.rx.recv().await {
            match self.extract_metadata(&path).await {
                Ok(media_item) => {
                    if self.tx.send(media_item).await.is_err() {
                        break;
                    }
                }
                Err(err) => {
                    tracing::warn!("Failed to analyze {:?}: {}", path, err);
                }
            }
        }
    }

    async fn extract_metadata(&self, p0: &PathBuf) -> Result<MediaSourceItem, std::io::Error> {
        let item = MediaSourceItem {
            id: "".to_string(),
            location: "".to_string(),
            title: "".to_string(),
            media_type: MediaType::Unspecified,
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
        };
        Ok(item)
    }
}