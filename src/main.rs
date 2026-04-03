slint::include_modules!();

use crate::background::battery_gauge::StatusEvent;
use crate::background::input_handler::PreferencesCommand;
use crate::background::media_source::{FilterCommand, FindCommand, MediaSourceCommand};
use crate::background::player::{PlayerCommand, PlayerEvent};
use crate::background::scheduler::scheduler::SchedulerEvent;
use crate::config::Config;
use crate::navigation_event::NavigationEvent;
use crate::slint_utils::rust_items_to_slint_model;
use background::start_tokio_background_tasks;
use rodio::cpal::traits::HostTrait;
use rodio::{DeviceSinkBuilder, DeviceTrait, Source};
use slint::{Model, ModelRc, SharedString, ToSharedString, VecModel};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use std::{env, iter};
use std::fs::File;
use std::path::Path;
use std::thread::sleep;
use rodio::cpal::{BufferSize, DeviceId};
use tokio::sync::mpsc;
use tracing::Level;
use tracing_subscriber::fmt::writer::MakeWriterExt;

mod background;
mod database_wrapper;
mod entity;
mod migrator;
mod file_utils;
mod config;
mod slint_utils;
mod input_event;
mod navigation_event;



fn main() -> Result<(), slint::PlatformError> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .init();

    // let _ = playtest();
    let media_path = "media/";
    let storage_path = ".nanowave/";


    let (media_source_tx, media_source_rx) = tokio::sync::mpsc::unbounded_channel::<background::media_source::MediaSourceCommand>();
    let (player_cmd_tx, player_cmd_rx) = tokio::sync::mpsc::unbounded_channel::<background::player::PlayerCommand>();
    let (player_evt_tx, mut player_evt_rx) = mpsc::unbounded_channel::<PlayerEvent>();


    let player_cmd_tx_shared = Arc::new(player_cmd_tx.clone());
    let media_source_filter_tx = media_source_tx.clone();
    let media_source_find_tx = media_source_tx.clone();


    let (prefs_cmd_tx, mut prefs_cmd_rx) = mpsc::unbounded_channel::<PreferencesCommand>();
    let (navigation_evt_tx, mut navigation_evt_rx) = mpsc::unbounded_channel::<NavigationEvent>();
    let (status_evt_tx, mut status_evt_rx) = mpsc::unbounded_channel::<StatusEvent>();
    let (scheduler_evt_tx, scheduler_evt_rx) = mpsc::unbounded_channel::<SchedulerEvent>();



    /*
        let scheduler_evt_tx_arc = Arc::new(scheduler_evt_tx.clone());

        let scheduler_evt_tx_nav_goto = scheduler_evt_tx_arc.clone();
        let scheduler_evt_tx_nav_back = scheduler_evt_tx_arc.clone();
        let scheduler_evt_tx_nav_forward = scheduler_evt_tx_arc.clone();
        let scheduler_evt_tx_source_filter = scheduler_evt_tx_arc.clone();
        let scheduler_evt_tx_source_find = scheduler_evt_tx_arc.clone();;
        */

    // return playtest();

    start_tokio_background_tasks(Config::new(storage_path.into(), media_path.into()),
                                 media_source_rx,
                                 player_cmd_tx_shared,
                                 player_cmd_rx,
                                 player_evt_tx.clone(),
                                 navigation_evt_tx.clone(),
                                 prefs_cmd_tx.clone(),
                                 status_evt_tx.clone(),
                                 scheduler_evt_tx,
                                 scheduler_evt_rx
    );




    // slint::set_platform(Box::new(linuxkms_backend));

    /*
    let x = BackendSelector::new()
        .select();
    let y = BackendSelector::new().

     */
    /*
    let platform = slint::platform::set_platform(Box::new(



    ));
    if let Ok(linuxkms_backend) = linuxkms_backend_result {
        platform::set_platform(linuxkms_backend);

    }
    */


    /*
    // /dev/input/event1
    let linuxkms_backend_result = i_slint_backend_linuxkms::BackendBuilder::default().with_libinput_event_hook(Box::new(move |e| -> bool {
        println!("input hook");
        false
    })).build();
    if let Ok(linuxkms_backend) = linuxkms_backend_result {
        // platform::set_platform(linuxkms_backend);

    }
    */

    let ui = MainWindow::new()?;
    let ui_weak = ui.as_weak();

    let ui_slint_navigation_goto = ui_weak.clone();
    let ui_slint_navigation_back = ui_weak.clone();
    // let ui_slint_navigation_forward = ui_weak.clone();

    let ui_slint_media_source_filter = ui_weak.clone();
    let ui_slint_media_source_find = ui_weak.clone();

    /*
    let scheduler_evt_tx = scheduler_evt_tx_arc.clone();

    let slint_preferences = ui.global::<SlintPreferences>();
    slint_preferences.on_user_interaction(move || {
        println!("user interaction detected: sending timer reset");
        let _ = scheduler_evt_tx.send(SchedulerEvent::Reset(type_name::<DisplayAutoShutdownTask>().to_string()));
    });
    */


    let navigation = ui.global::<SlintNavigation>();
    navigation.on_path(move |value| {
        let ui = ui_slint_navigation_goto.upgrade().unwrap();
        let nav = ui.global::<SlintNavigation>();
        nav.set_route(value);
        let history_item = nav.get_route();

        let tmp_next_index = nav.get_history_index() + 1;
        let next_index = if tmp_next_index > 1000 {
            1000
        } else {
            tmp_next_index
        };
        let skip = if tmp_next_index > 1000 { 1 } else { 0 };
        let take = next_index - skip;
        let vec_of_history: Vec<ModelRc<SharedString>> = nav
            .get_history()
            .iter()
            .skip(skip as usize)
            .take(take as usize)
            .chain(iter::once(history_item))
            .collect();
        let history = VecModel::from(vec_of_history);
        nav.set_history(ModelRc::new(history));
        nav.set_history_index(next_index);
    });

    // let scheduler_evt_tx = scheduler_evt_tx_arc.clone();
    navigation.on_offset(move |offset| {

        let ui = ui_slint_navigation_back.upgrade().unwrap();
        let nav = ui.global::<SlintNavigation>();
        let new_index = nav.get_history_index() + offset;
        let new_index_usize = new_index as usize;
        let vec_of_history: Vec<ModelRc<SharedString>> = nav.get_history().iter().collect();

        if vec_of_history.get(new_index_usize).is_some() {
            nav.set_route(vec_of_history[new_index_usize].clone());
            nav.set_history_index(new_index);
        }
    });


    // let scheduler_evt_tx = scheduler_evt_tx_arc.clone();

    let slint_media_source = ui.global::<SlintMediaSource>();
    slint_media_source.on_filter({
        println!("main.rs -> on_filter");
        let ui = ui_slint_media_source_filter.upgrade().unwrap();
        move |query| {
            let ui_weak_clone = ui.as_weak().clone();
            let cmd = MediaSourceCommand::Filter(FilterCommand {
                query: query.to_string(),
                callback: Box::new(|items| {
                    println!("Filtercommand::callback is called");

                    slint::invoke_from_event_loop(move || {
                        let Some(ui) = ui_weak_clone.upgrade() else {
                            return;
                        };

                        let media_source = ui.global::<SlintMediaSource>();
                        media_source.set_filter_results(
                            rust_items_to_slint_model(items, false),
                        );

                        media_source.set_is_loading(false);
                    }).unwrap();
                }),
            });

            let media_source = ui.global::<SlintMediaSource>();
            media_source.set_is_loading(true);
            media_source.set_find_results(ModelRc::default());
            media_source_filter_tx.send(cmd).ok();
            println!("main.rs -> on_filter end");
        }
    });



    slint_media_source.on_find({


        let ui = ui_slint_media_source_find.upgrade().unwrap();
        move |id| {
            let ui_weak_clone = ui.as_weak().clone();
            let cmd = MediaSourceCommand::Find(FindCommand {
                id: id.to_string(),
                callback: Box::new(|item_option| {
                    slint::invoke_from_event_loop(move || {
                        let Some(ui) = ui_weak_clone.upgrade() else {
                            return;
                        };

                        let media_source = ui.global::<SlintMediaSource>();
                        if let Some(item) = item_option {
                            media_source.set_find_results(
                                rust_items_to_slint_model(vec![item], true),
                            );
                        } else {
                            media_source.set_find_results(ModelRc::default());
                        }
                        media_source.set_is_loading(false);
                    }).unwrap();
                }),
            });

            let media_source = ui.global::<SlintMediaSource>();
            media_source.set_is_loading(true);
            media_source.set_find_results(ModelRc::default());
            media_source_find_tx.send(cmd).ok();
        }
    });





    let slint_audio_player = ui.global::<SlintAudioPlayer>();
    slint_audio_player.on_play_test({
        let tx = player_cmd_tx.clone();
        move || {
            println!("slint_audio_player.on_play_test before");
            tx.send(PlayerCommand::PlayTest()).unwrap();
            println!("slint_audio_player.on_play_test after");
        }
    });


    slint_audio_player.on_play_media({
        let tx = player_cmd_tx.clone();
        move |media_item_id: SharedString, position: i64| {
            let position_as_duration = Duration::from_millis(position as u64);
            tx.send(PlayerCommand::PlayMedia(media_item_id.to_string(), position_as_duration))
                .unwrap();
        }
    });

    slint_audio_player.on_play({
        let tx = player_cmd_tx.clone();
        move || {
            tx.send(PlayerCommand::Play()).unwrap();
        }
    });

    slint_audio_player.on_pause({
        let tx = player_cmd_tx.clone();
        move || {
            tx.send(PlayerCommand::Pause()).unwrap();
        }
    });

    slint_audio_player.on_toggle({
        let tx = player_cmd_tx.clone();
        move || {
            tx.send(PlayerCommand::Toggle()).unwrap();
        }
    });

    slint_audio_player.on_next({
        let tx = player_cmd_tx.clone();
        move || {
            tx.send(PlayerCommand::Next()).unwrap();
        }
    });

    slint_audio_player.on_previous({
        let tx = player_cmd_tx.clone();
        move || {
            tx.send(PlayerCommand::Previous()).unwrap();
        }
    });

    slint_audio_player.on_seek_relative({
        let tx = player_cmd_tx.clone();
        move |millis_i64| {
            tx.send(PlayerCommand::SeekRelative(millis_i64)).unwrap();
        }
    });

    slint_audio_player.on_seek_to({
        let tx = player_cmd_tx.clone();
        move |millis_i64: i64| {
            tx.send(PlayerCommand::SeekTo(Duration::from_millis(
                millis_i64 as u64,
            )))
                .unwrap();
        }
    });

    /*
        let ui_handle_prefs = ui.as_weak();
        slint::spawn_local(async move {
            while let Some(event) = prefs_cmd_rx.recv().await {
                if let Some(ui) = ui_handle_prefs.upgrade() {
                    // let inner = ui.global::<SlintPreferences>();
                    match event {

                        PreferencesCommand::SetEnableTouchEvents(enable_touch_events) => {
                            println!("set_enable_touch_events: {}", enable_touch_events);
                            // inner.set_enable_touch_events(enable_touch_events);
                        }


                    }

                }
            }
        }).unwrap();


        let ui_handle_status = ui.as_weak();
        slint::spawn_local(async move {
            while let Some(event) = status_evt_rx.recv().await {
                if let Some(ui) = ui_handle_status.upgrade() {
                    let inner = ui.global::<SlintStatus>();
                    match event {
                        StatusEvent::UpdateBattery(_percentage) => {
                            inner.set_battery(SlintBatteryStatus {
                                percent: 0.82,
                                charging: true,
                                health: "Good".into(),
                            });
                        }
                    }

                }
            }
        }).unwrap();
    */
    let ui_handle_navigation = ui.as_weak();
    slint::spawn_local(async move {
        while let Some(event) = navigation_evt_rx.recv().await {
            if let Some(ui) = ui_handle_navigation.upgrade() {
                let inner = ui.global::<SlintNavigation>();
                match event {
                    NavigationEvent::NavigateTo(path) => {
                        let new_path_vec: Vec<SharedString> = path.into_iter().map(Into::into).collect();
                        let new_path = ModelRc::new(VecModel::from(new_path_vec));
                        // todo this has been removed for debugging
                        inner.invoke_path(new_path);
                    }
                }

            }
        }
    }).unwrap();


    let ui_handle_player = ui.as_weak();
    slint::spawn_local(async move {

        while let Some(event) = player_evt_rx.recv().await {
            if let Some(ui) = ui_handle_player.upgrade() {
                let inner = ui.global::<SlintAudioPlayer>();

                match event {
                    PlayerEvent::Status(item, status) => {
                        // inner.set_current_item_id(item_id.to_shared_string());

                        let slint_items = rust_items_to_slint_model(vec![item], true);
                        if let Some(item) = slint_items.row_data(0) {
                            inner.set_current_item(item);
                        }
                        inner.set_status(status.to_shared_string());

                    }

                    PlayerEvent::Stopped => {}

                    PlayerEvent::Position(_item_id, position) => {
                        // println!("item_id: {}, position: {:?}", item_id, position.clone());
                        // inner.set_current_item_id(item_id.to_shared_string());

                        let mut item = inner.get_current_item();
                        item.position_formatted = format_duration(position.clone()).to_shared_string();
                        inner.set_current_item(item);




                        // inner.get_current_item().set_position_formatted(format_duration(position).to_shared_string());
                        // inner.set_position_formatted(format_duration(position).to_shared_string());
                    }
                }
            } else {
                // UI was dropped; stop listening
                break;
            }
        }
    }).unwrap();

    ui.run()
}



pub fn format_duration(duration: Duration) -> String {
    let millis = duration.as_millis();
    let secs = millis / 1000;
    let h = secs / (60 * 60);
    let m = (secs / 60) % 60;
    let s = secs % 60;
    format!("{:0>2}:{:0>2}:{:0>2}", h, m, s)
}


fn playtest() -> Result<(), slint::PlatformError>{

    let device_ids = match env::var("NANOWAVE_AUDIO_DEVICE") {
        Ok(val) => vec![val],
        Err(_e) => vec![],
    };

    let audio_file = match env::var("NANOWAVE_AUDIO_FILE") {
        Ok(val) => val,
        Err(_e) => "media/sample-3s.wav".to_string()
    };

    let host = rodio::cpal::default_host();

    let device_id_strings = device_ids.to_vec();
    let mut device_option = host.default_output_device();
    let device_ids: Vec<DeviceId> = device_id_strings
        .into_iter()
        .filter_map(|s| DeviceId::from_str(&s).ok())
        .collect();

    for device_id in device_ids {
        print!("attempt get device by id: {}", device_id);
        let dev = host.device_by_id(&device_id);
        if dev.is_some() {
            println!(" => success");

            device_option = dev;
            break;
        } else {
            println!(" => failed");
        }
    }


    print!("trying to connect audio device");
    if let Some(device) = device_option {
        println!(" => device {:?} is available", device.id());

        if let Ok(builder) = DeviceSinkBuilder::from_device(device)
            && let Ok(stream) = builder.with_buffer_size(BufferSize::Fixed(2048)).open_stream(){
            println!("sandreas: builder.stream.buffer_size: {:?}", stream.config().buffer_size());

            let sink = rodio::Player::connect_new(stream.mixer());

            sink.clear();
            let waves = vec![230f32, 270f32, 330f32, 270f32, 230f32];
            for w in waves {
                let source = rodio::source::SineWave::new(w).amplify(0.1);
                sink.append(source);
                sink.play();
                sleep(Duration::from_millis(200));
                sink.stop();
                sink.clear();
            }

            let location = audio_file.as_str();

            let path = Path::new(location);
            let file_result = File::open(path);

            if let Ok(file) = file_result {
                let decoder_result = rodio::Decoder::try_from(file);
                if let Ok(decoder) = decoder_result{
                    sink.clear();
                    sink.append(decoder);
                    sink.play();
                    sink.sleep_until_end();
                } else {
                    println!("error decoding");
                }
            } else {
                println!("error file result");
            }

        } else {
            println!("failed to open audio stream");
        }
    } else {
        println!("failed to find audio device");
    }

    Ok(())
}
