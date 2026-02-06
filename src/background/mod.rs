use crate::background::database_existence_checker::DatabaseExistenceChecker;
use crate::background::database_updater::DatabaseUpdater;
use crate::background::database_upsert_item::DatabaseUpsertItem;
use crate::background::file_media_source::FileMediaSource;
use crate::background::file_scanner::{extension_filter, FileScanner, FileScannerAction};
use crate::background::headset_handler::HeadsetHandler;
use crate::background::input_handler::InputHandler;
use crate::background::media_source::MediaSourceCommand;
use crate::background::metadata_retriever::MetadataRetriever;
use crate::background::player::{Player, PlayerCommand, PlayerEvent};
use crate::config::Config;
use crate::database_wrapper::DatabaseWrapper;
use crate::input_event;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use crate::background::gpio_handler::GpioHandler;
use crate::background::gpio_pin::GpioPin;


mod file_scanner;
mod database_existence_checker;
mod database_updater;
mod metadata_retriever;
mod database_upsert_item;
mod file_media_source;
pub(crate) mod media_source;
pub(crate) mod player;
mod input_handler;
mod headset_handler;
mod gpio_handler;
mod gpio_pin;

pub fn start_tokio_background_tasks(config: Config,
                                    media_source_rx: UnboundedReceiver<MediaSourceCommand>,
                                    player_tx: Arc<UnboundedSender<PlayerCommand>>,
                                    player_rx: UnboundedReceiver<PlayerCommand>,
                                    player_evt_tx: UnboundedSender<PlayerEvent>,
) {
    println!("=== start_tokio_background_tasks");
    thread::spawn(move || {
        tokio::runtime::Runtime::new().unwrap().block_on(background_tasks(
            &config,
            media_source_rx,
            player_tx,
            player_rx,
            player_evt_tx,
        ));
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

pub async fn background_tasks(config: &Config,
                              media_source_rx: UnboundedReceiver<MediaSourceCommand>,
                              player_tx: Arc<UnboundedSender<PlayerCommand>>,
                              player_rx: UnboundedReceiver<PlayerCommand>,
                              player_evt_tx: UnboundedSender<PlayerEvent>,
) {

    println!("=== background_tasks");


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
    let (input_event_tx, input_event_rx) = mpsc::unbounded_channel::<input_event::InputEvent>();

    let headset_tx = Arc::new(input_event_tx.clone());
    let gpio_tx = Arc::new(input_event_tx.clone());

    let config_file_scanner = config.clone();
    let file_scanner_task = tokio::spawn(async {
        println!("=== file_scanner_task");

        let filter = extension_filter(vec!["mp3", "flac", "wav", "m4b"]);
        let _ = FileScanner::new(config_file_scanner, file_scanner_rx, file_tx).scan_files(filter).await;
    });


    let database_checker_task = tokio::spawn(async {
        println!("=== database_checker_task");
        let _ = DatabaseExistenceChecker::new(db_database_checker, file_rx, db_checker_tx).check_items_for_needed_update().await;
    });

    let config_meta = config.clone();
    let metadata_retriever_task = tokio::spawn(async {
        println!("=== metadata_retriever_task");
        let _ = MetadataRetriever::new(config_meta, db_checker_rx, meta_retriever_tx).retrieve_metadata().await;
    });

    let database_updater_task = tokio::spawn(async {
        println!("=== database_updater_task");
        let _ = DatabaseUpdater::new(db, meta_retriever_rx).update_items().await;
    });

    let media_source = FileMediaSource::new(config.clone(), db_media_source);
    let media_source_player = media_source.clone();
    let media_source_task = tokio::spawn(async {
        println!("=== media_source_task");
        let _ = media_source.run(media_source_rx).await;
    });
    
    let player_tx_player = player_tx.clone();
    let player_task = tokio::spawn(async {
        println!("=== player_task");
        let preferred_device = "USB-C to 3.5mm Headphone Jack A".to_string();
        let fallback_device = "pipewire".to_string();

        let _ = Player::new(Arc::new(media_source_player), preferred_device, fallback_device).run(player_tx_player, player_rx, player_evt_tx).await;
    });



    let headset_task = tokio::spawn(async {
        println!("=== headset_task");
        let device_paths = vec!["/dev/input/event2", "/dev/input/event13"];
        let _ = HeadsetHandler::new().run(device_paths, headset_tx).await;
    });



    let player_tx_input_handler = player_tx.clone();
    let input_handler_task = tokio::spawn(async {
        println!("=== input_handler_task");
        let _ = InputHandler::new().run(input_event_rx, player_tx_input_handler).await;
    });


    let gpio_pins = vec![GpioPin::A22, GpioPin::A23, GpioPin::A24, GpioPin::A25];
    let gpio_task = tokio::spawn(async {
        println!("=== gpio_task");
        let _ = GpioHandler::new().run(gpio_pins, gpio_tx).await;
    });


    println!("=== file_scanner_tx.send");
    let _ = file_scanner_tx.send(FileScannerAction::ScanFiles).await;
    
    let _ = tokio::join!(file_scanner_task,
        database_checker_task,
        metadata_retriever_task,
        database_updater_task,
        media_source_task,
        player_task,
        headset_task,
        gpio_task,
        input_handler_task
    );
}