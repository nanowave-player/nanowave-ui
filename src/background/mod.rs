use crate::background::battery_gauge::{BatteryGauge, StatusEvent};
use crate::background::database_existence_checker::DatabaseExistenceChecker;
use crate::background::database_updater::DatabaseUpdater;
use crate::background::database_upsert_item::DatabaseUpsertItem;
use crate::background::display_controller::{DisplayCommand, DisplayController};
use crate::background::file_media_source::FileMediaSource;
use crate::background::file_scanner::{extension_filter, FileScanner, FileScannerAction};
use crate::background::gpio_handler::GpioHandler;
use crate::background::gpio_pin::GpioPin;
use crate::background::headset_handler::HeadsetHandler;
use crate::background::input_handler::{InputHandler, PreferencesCommand};
use crate::background::media_source::{MediaSource, MediaSourceCommand};
use crate::background::metadata_retriever::MetadataRetriever;
use crate::background::player::{Player, PlayerCommand, PlayerEvent};
use crate::background::scheduler::display_auto_shudown_task::DisplayAutoShutdownTask;
use crate::background::scheduler::scheduler::{Scheduler, SchedulerEvent};
use crate::background::touch_handler::TouchHandler;
use crate::config::Config;
use crate::database_wrapper::DatabaseWrapper;
use crate::input_event;
use crate::navigation_event::NavigationEvent;
use std::path::PathBuf;
use std::sync::Arc;
use std::{env, thread};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

mod file_scanner;
mod database_existence_checker;
mod database_updater;
mod metadata_retriever;
mod database_upsert_item;
mod file_media_source;

mod headset_handler;
mod gpio_handler;
mod gpio_pin;
mod file_media_source_playback_history;
mod display_controller;

pub(crate) mod battery_gauge;
pub(crate) mod input_handler;
pub(crate) mod media_source;
pub(crate) mod player;
pub(crate) mod scheduler;
mod touch_handler;

pub fn start_tokio_background_tasks(config: Config,
                                    media_source_rx: UnboundedReceiver<MediaSourceCommand>,
                                    player_tx: Arc<UnboundedSender<PlayerCommand>>,
                                    player_rx: UnboundedReceiver<PlayerCommand>,
                                    player_evt_tx: UnboundedSender<PlayerEvent>,
                                    navigation_evt_tx: UnboundedSender<NavigationEvent>,
                                    prefs_tx: UnboundedSender<PreferencesCommand>,
                                    status_tx: UnboundedSender<StatusEvent>,
                                    scheduler_tx: UnboundedSender<SchedulerEvent>,
                                    scheduler_rx: UnboundedReceiver<SchedulerEvent>,

) {
    println!("=== start_tokio_background_tasks");
    thread::spawn(move || {
        tokio::runtime::Runtime::new().unwrap().block_on(background_tasks(
            &config,
            media_source_rx,
            player_tx,
            player_rx,
            player_evt_tx,
            navigation_evt_tx,
            prefs_tx,
            status_tx,
            scheduler_tx,
            scheduler_rx
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
                              navigation_evt_tx: UnboundedSender<NavigationEvent>,
                              _prefs_tx: UnboundedSender<PreferencesCommand>,
                              status_tx: UnboundedSender<StatusEvent>,
                              scheduler_tx: UnboundedSender<SchedulerEvent>,
                              scheduler_rx: UnboundedReceiver<SchedulerEvent>,

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
    let (display_tx, display_rx) = mpsc::unbounded_channel::<DisplayCommand>();
    // let (scheduler_tx, scheduler_rx) = mpsc::unbounded_channel::<SchedulerEvent>();

    let headset_tx = Arc::new(input_event_tx.clone());
    let gpio_tx = Arc::new(input_event_tx.clone());


    let display_tx_scheduler = display_tx.clone();
    let scheduler_task = tokio::spawn(async {
        let _ = Scheduler::new()
            .add_task(Box::new(DisplayAutoShutdownTask::new(display_tx_scheduler, 60_000)))
            .run(scheduler_rx).await;
    });

    let battery_checker_task = tokio::spawn(async {
        let _ = BatteryGauge::new().run(status_tx).await;
    });

    let scheduler_tx_display = scheduler_tx.clone();
    let display_controller_task = tokio::spawn(async {
        let _ = DisplayController::new().run(scheduler_tx_display, display_rx).await;
    });


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
    let media_source_history = media_source.clone();

    let media_source_task = tokio::spawn(async {
        println!("=== media_source_task");
        let _ = media_source.run(media_source_rx).await;
    });



    let player_tx_player = player_tx.clone();
    let player_task = tokio::spawn(async {
        println!("=== player_task");



        let device_ids = match env::var("NANOWAVE_AUDIO_DEVICE") {
            Ok(val) => vec![val],
            Err(_e) => vec![],
        };

        if device_ids.is_empty() {
            println!("NANOWAVE_AUDIO_DEVICE environment variable is not set, using default audio device");
        } else {
            println!("NANOWAVE_AUDIO_DEVICE={:?}", device_ids);
        }

        let _ = Player::new(Arc::new(media_source_player), device_ids).run(player_tx_player, player_rx, player_evt_tx).await;
    });




    let player_tx_input_handler = player_tx.clone();
    let input_handler_task = tokio::spawn(async {
        println!("=== input_handler_task");
        let _ = InputHandler::new().run(input_event_rx, player_tx_input_handler, display_tx).await;
    });


    let gpio_pins = vec![GpioPin::A22, GpioPin::A23, GpioPin::A24, GpioPin::A25];
    let gpio_task = tokio::spawn(async {
        println!("=== gpio_task");
        let _ = GpioHandler::new().run(gpio_pins, gpio_tx).await;
    });


    println!("=== file_scanner_tx.send");
    let _ = file_scanner_tx.send(FileScannerAction::ScanFiles).await;

    // todo:
    // - this probably should be abstracted in the media_source and not be tightly coupled to the sqlite database
    // - approach:
    //   - media_source.history_query(query:String) -> Vec<MediaSourceHistoryItem>
    //   - media_source.history_update(item: MediaSourceItem, position: Duration)
    //   - query is the same as in media_source.filter(query: String) and can be used to limit
    //   - idea is to use a lisp-like syntax: (limit (where (eq id "100") 0 10))
    // let playback_history = PlaybackHistory::new(db_playback_history);
    // let last_played_item_model_option = playback_history.load_last_history_item().await;

    let player_tx_history = player_tx.clone();

    if let Some(last_played_history_item) =  media_source_history.history_latest().await {
        let item_id = last_played_history_item.item.id.clone();
        let _ = player_tx_history.send(PlayerCommand::RestoreLastSession(last_played_history_item));
        let _ = navigation_evt_tx.send(NavigationEvent::NavigateTo(vec!["details".into(), item_id.clone()]));
    }

    let touch_task = tokio::spawn(async {
        println!("=== touch_task");
        let device_paths = vec!["/dev/input/event1", "/dev/input/event12"];
        let _ = TouchHandler::new().run(device_paths, scheduler_tx).await;
    });


    // start this at last pos because it is blocking, if there is no headset connected (todo: fix this)
    let headset_task = tokio::spawn(async {
        println!("=== headset_task");
        let device_paths = vec!["/dev/input/event2", "/dev/input/event13"];
        let _ = HeadsetHandler::new().run(device_paths, headset_tx).await;
    });






    let _ = tokio::join!(
        display_controller_task,
        file_scanner_task,
        database_checker_task,
        metadata_retriever_task,
        database_updater_task,
        media_source_task,
        player_task,
        gpio_task,
        input_handler_task,
        battery_checker_task,
        touch_task,
        headset_task,
        scheduler_task,
    );
}


