use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub enum FileScannerAction {
    ScanFiles
}


pub struct FileScanner {
    media_path: String,
    rx: tokio::sync::mpsc::Receiver<FileScannerAction>,
    tx: tokio::sync::mpsc::Sender<PathBuf>
}

impl FileScanner {
    pub fn new(media_path:String, rx: tokio::sync::mpsc::Receiver<FileScannerAction>, tx: tokio::sync::mpsc::Sender<PathBuf>) -> Self {
        Self {
            media_path,
            rx,
            tx
        }
    }
    pub async fn scan_files<F>(
        &mut self, filter: F
    ) -> anyhow::Result<()> where
        F: Fn(&Path) -> bool + Send + Sync,{

        let self_root = PathBuf::from(self.media_path.clone());
        
        
        while let Some(action) = self.rx.recv().await {
            println!("filescanner: received action");
            match action {
                FileScannerAction::ScanFiles => {

                    let root = self_root.clone();
                    for entry in walkdir::WalkDir::new(root) {
                        println!("filescanner: walk entry");

                        let entry = entry?;
                        if entry.file_type().is_file() && filter(entry.path()) {
                            println!("filescanner: send entry");
                            if let Err(_) = self.tx.send(entry.path().to_path_buf()).await {
                                break; // downstream closed
                            }
                        }
                    }
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