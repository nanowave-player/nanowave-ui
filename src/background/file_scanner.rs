use std::path::PathBuf;

pub struct FileScanner {
    root: PathBuf,
    tx: tokio::sync::mpsc::Sender<PathBuf>
}

impl FileScanner {
    pub fn new(root: PathBuf, tx: tokio::sync::mpsc::Sender<PathBuf>) -> Self {
        Self {
            root,
            tx
        }
    }
    pub async fn scan_files(
        &self
    ) -> anyhow::Result<()> {
        let root = self.root.clone();
        for entry in walkdir::WalkDir::new(root) {
            let entry = entry?;
            if entry.file_type().is_file() {
                if let Err(_) = self.tx.send(entry.path().to_path_buf()).await {
                    break; // downstream closed
                }
            }
        }
        Ok(())
    }


}