use crate::background::database_existence_checker::{DatabaseExistenceChecker};
use crate::background::file_scanner::{extension_filter, FileScanner};
use crate::database_wrapper::DatabaseWrapper;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use crate::background::database_upsert_item::DatabaseUpsertItem;
use crate::background::metadata_retriever::MetadataRetriever;

mod file_scanner;
mod media_analyzer;
mod database_existence_checker;
mod database_updater;
mod metadata_retriever;
mod database_upsert_item;

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
    let base_path_clone = PathBuf::from(base_path_str.to_string().clone());
    // let base_path_clone = base_path.clone();

    let db_result = DatabaseWrapper::new(base_path_str.to_string()).connect().await;
    if db_result.is_err() {
        println!("Connection to database failed {:?}", db_result);
        return;
    }

    let db = db_result.unwrap();
    let db_database_checker = db.clone();

    let (file_tx, file_rx) = tokio::sync::mpsc::channel::<PathBuf>(100);
    let (db_checker_tx, db_checker_rx) = tokio::sync::mpsc::channel::<DatabaseUpsertItem>(100);
    let (meta_retriever_tx, meta_retriever_rx) = tokio::sync::mpsc::channel::<DatabaseUpsertItem>(100);


    // let (media_tx, media_rx) = tokio::sync::mpsc::channel::<MediaSourceItem>(100);

    let file_scanner_task = tokio::spawn(async {
        let base_path_string = base_path.into_os_string().into_string().unwrap();
        let base_path = PathBuf::from(base_path_string);
        let file_scanner = FileScanner::new(base_path, file_tx);
        let filter = extension_filter(vec!["mp3", "flac", "wav"]);
        file_scanner.scan_files(filter).await
    });

    let database_checker_task = tokio::spawn(async {
        let mut database_checker = DatabaseExistenceChecker::new(db_database_checker, file_rx, db_checker_tx);
        database_checker.check_items_for_needed_update().await
    });

    let metadata_retriever_task = tokio::spawn(async {
        let base_path_string = base_path_clone.into_os_string().into_string().unwrap();
        let mut metadata_retriever = MetadataRetriever::new(base_path_string.clone().to_string(), meta_retriever_rx, meta_retriever_tx);
        metadata_retriever.retrieve_metadata().await
    });

    let _ = tokio::join!(file_scanner_task, database_checker_task, metadata_retriever_task);


    // better approach
    // DatabaseChecker
    // MetadataRetriever
    // DatabaseUpdater


/*

    tokio::spawn(async {
        let mut media_analyzer = MediaAnalyzer::new(file_rx, media_tx);
        media_analyzer.analyze_metadata().await
    });
*/
/*

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

 */
}