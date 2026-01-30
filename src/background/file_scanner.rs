use std::collections::HashSet;
use std::path::{Path, PathBuf};

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
    pub async fn scan_files<F>(
        &self, filter: F
    ) -> anyhow::Result<()> where
        F: Fn(&Path) -> bool + Send + Sync,{
        let root = self.root.clone();
        for entry in walkdir::WalkDir::new(root) {
            let entry = entry?;
            if entry.file_type().is_file() && filter(entry.path()) {
                if let Err(_) = self.tx.send(entry.path().to_path_buf()).await {
                    break; // downstream closed
                }
            }
        }
        Ok(())
    }
}



pub fn extension_filter(exts: Vec<&'static str>) -> impl Fn(&Path) -> bool {
    let allowed: HashSet<String> =
        exts.into_iter().map(|e| e.to_ascii_lowercase()).collect();

    move |path: &Path| {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| allowed.contains(&e.to_ascii_lowercase()))
            .unwrap_or(false)
    }
}