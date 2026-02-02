use crate::background::database_existence_checker::DatabaseExistenceChecker;
use crate::background::database_updater::DatabaseUpdater;
use crate::background::database_upsert_item::DatabaseUpsertItem;
use crate::background::file_scanner::{extension_filter, FileScanner, FileScannerAction};
use crate::background::metadata_retriever::MetadataRetriever;
use crate::database_wrapper::DatabaseWrapper;
use std::path::PathBuf;
use std::thread;
use tokio::sync::mpsc::UnboundedReceiver;
use crate::background::file_media_source::FileMediaSource;
use crate::background::media_source::MediaSourceCommand;
use crate::config::Config;

mod file_scanner;
mod database_existence_checker;
mod database_updater;
mod metadata_retriever;
mod database_upsert_item;
mod file_media_source;
pub(crate) mod media_source;

pub fn start_tokio_background_tasks(config: Config, media_source_rx: UnboundedReceiver<MediaSourceCommand>) {
    thread::spawn(move || {
        tokio::runtime::Runtime::new().unwrap().block_on(background_tasks(&config, media_source_rx));
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

pub async fn background_tasks(config: &Config, media_source_rx: UnboundedReceiver<MediaSourceCommand>) {

    // let base_path_str = config.base_path.clone();
    // let base_path = PathBuf::from(base_path_str.to_string().clone());
    // let base_path_clone = PathBuf::from(base_path_str.to_string().clone());
    // let base_path_clone = base_path.clone();

    let db_base_path = config.base_path.clone();
    let db_result = DatabaseWrapper::new(db_base_path).connect().await;
    if db_result.is_err() {
        println!("Connection to database failed {:?}", db_result);
        return;
    }

    let db = db_result.unwrap();
    let db_database_checker = db.clone();
    let db_media_source = db.clone();

    let (file_scanner_tx, file_scanner_rx) = tokio::sync::mpsc::channel::<FileScannerAction>(100);
    let (file_tx, file_rx) = tokio::sync::mpsc::channel::<PathBuf>(100);
    let (db_checker_tx, db_checker_rx) = tokio::sync::mpsc::channel::<DatabaseUpsertItem>(100);
    let (meta_retriever_tx, meta_retriever_rx) = tokio::sync::mpsc::channel::<DatabaseUpsertItem>(100);


    // let (media_tx, media_rx) = tokio::sync::mpsc::channel::<MediaSourceItem>(100);
    let config_file_scanner = config.clone();
    let file_scanner_task = tokio::spawn(async {
        // let base_path_string = base_path.into_os_string().into_string().unwrap();
        // let base_path = PathBuf::from(base_path_string);
        let filter = extension_filter(vec!["mp3", "flac", "wav", "m4b"]);
        let _ = FileScanner::new(config_file_scanner, file_scanner_rx, file_tx).scan_files(filter).await;
    });


    let database_checker_task = tokio::spawn(async {
        let _ = DatabaseExistenceChecker::new(db_database_checker, file_rx, db_checker_tx).check_items_for_needed_update().await;
    });

    let config_meta = config.clone();
    let metadata_retriever_task = tokio::spawn(async {
        // let base_path_string = base_path_clone.into_os_string().into_string().unwrap();
        let _ = MetadataRetriever::new(config_meta, db_checker_rx, meta_retriever_tx).retrieve_metadata().await;
    });

    let database_updater_task = tokio::spawn(async {
        let _ = DatabaseUpdater::new(db, meta_retriever_rx).update_items().await;
    });

    let config_media_source = config.clone();
    let media_source_task = tokio::spawn(async {
        let _ = FileMediaSource::new(config_media_source, db_media_source).run(media_source_rx).await;
    });

    let _ = file_scanner_tx.send(FileScannerAction::ScanFiles).await;

    let _ = tokio::join!(file_scanner_task, database_checker_task, metadata_retriever_task, database_updater_task, media_source_task);
}