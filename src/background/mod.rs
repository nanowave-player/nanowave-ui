use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use media_source::media_source_item::MediaSourceItem;
use crate::background::database_updater::DatabaseUpdater;
use crate::background::file_scanner::{extension_filter, FileScanner};
use crate::background::media_analyzer::MediaAnalyzer;

mod file_scanner;
mod media_analyzer;
mod database_updater;

pub fn start_tokio_background_tasks(base_path_str: &str) {
    let base_path_string = base_path_str.to_string().clone();
    thread::spawn(move || {
        tokio::runtime::Runtime::new().unwrap().block_on(background_tasks(base_path_string.as_str()));
    });

    /*
    // spawn multiple workers
    let workers = num_cpus::get();

    for _ in 0..workers {
        let rx = file_rx.clone();
        let tx = media_tx.clone();
        tokio::spawn(analyze_metadata(rx, tx));
    }

    drop(media_tx); // important: close channel when workers finish
    */
}

pub async fn background_tasks(base_path_str: &str) {
    let base_path = PathBuf::from(base_path_str.to_string().clone());

    let (file_tx, file_rx) = tokio::sync::mpsc::channel::<PathBuf>(100);
    let (media_tx, media_rx) = tokio::sync::mpsc::channel::<MediaSourceItem>(100);

    tokio::spawn(async {
        let file_scanner = FileScanner::new(base_path, file_tx);
        let filter = extension_filter(vec!["mp3", "flac", "wav"]);
        file_scanner.scan_files(filter).await
    });


    tokio::spawn(async {
        let mut media_analyzer = MediaAnalyzer::new(file_rx, media_tx);
        media_analyzer.analyze_metadata().await
    });


    tokio::spawn(async {
        // let mut database_updater = DatabaseUpdater::new(db, media_rx);
        // database_updater.update_database().await
    });


    let countertask1 = tokio::spawn(async {
        let mut counter = 0;
        loop {
            counter = counter+1;
            println!("countertask 1: {}", counter);
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    let countertask2 = tokio::spawn(async {
        let mut counter = 0;
        loop {
            counter = counter+1;
            println!("countertask 2: {}", counter);
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    let _ = tokio::join!(countertask1, countertask2);
}