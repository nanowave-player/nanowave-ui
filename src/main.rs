slint::include_modules!();

use crate::background::media_source::{FilterCommand, FindCommand, MediaSourceCommand};
use crate::background::player::{PlayerCommand, PlayerEvent};
use crate::config::Config;
use background::start_tokio_background_tasks;
use slint::{Model, ModelRc, SharedString, ToSharedString, VecModel};
use std::iter;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

mod background;
mod database_wrapper;
mod entity;
mod migrator;
mod file_utils;
mod config;
mod slint_utils;
mod input_event;

fn main() -> Result<(), slint::PlatformError> {
    let base_path = "media/";
    let (media_source_tx, media_source_rx) = tokio::sync::mpsc::unbounded_channel::<background::media_source::MediaSourceCommand>();
    let (player_cmd_tx, player_cmd_rx) = tokio::sync::mpsc::unbounded_channel::<background::player::PlayerCommand>();
    let (player_evt_tx, mut player_evt_rx) = mpsc::unbounded_channel::<PlayerEvent>();

    let player_cmd_tx_shared = Arc::new(player_cmd_tx.clone());

    start_tokio_background_tasks(Config::new(base_path.to_string()),
                                 media_source_rx,
                                 player_cmd_tx_shared,
                                 player_cmd_rx,
                                 player_evt_tx.clone(),
    );

    let ui = MainWindow::new()?;
    let ui_weak = ui.as_weak();

    let ui_slint_media_source_filter = ui_weak.clone();
    let ui_slint_media_source_find = ui_weak.clone();

    let navigation = ui.global::<SlintNavigation>();
    let ui_nav = ui_weak.clone();
    navigation.on_goto(move |value| {
        let ui = ui_nav.upgrade().unwrap();
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

    let ui_back = ui_weak.clone();
    navigation.on_back(move || {
        let ui = ui_back.upgrade().unwrap();
        let nav = ui.global::<SlintNavigation>();
        let current_index = nav.get_history_index();
        let vec_index = current_index as usize;
        let vec_of_history: Vec<ModelRc<SharedString>> = nav.get_history().iter().collect();
        if current_index == 0 || vec_of_history.is_empty() {
            return;
        }
        nav.set_route(vec_of_history[vec_index - 1].clone());
        nav.set_history_index(current_index - 1);
    });

    let ui_forward = ui_weak.clone();
    navigation.on_forward(move || {
        let ui = ui_forward.upgrade().unwrap();
        let nav = ui.global::<SlintNavigation>();
        let current_index = nav.get_history_index();
        let vec_index = current_index as usize;
        let vec_of_history: Vec<ModelRc<SharedString>> = nav.get_history().iter().collect();
        if vec_of_history.len() < vec_index + 2 {
            return;
        }
        nav.set_route(vec_of_history[vec_index + 1].clone());
        nav.set_history_index(current_index + 1);
    });


    let slint_media_source = ui.global::<SlintMediaSource>();

    let filter_tx = media_source_tx.clone();
    slint_media_source.on_filter({

        let ui = ui_slint_media_source_filter.upgrade().unwrap();
        move |query| {
            let ui_weak_find = ui.as_weak().clone();
            let cmd = MediaSourceCommand::Filter(FilterCommand {
                query: query.to_string(),
                callback: Box::new(|items| {
                    slint::invoke_from_event_loop(move || {
                        let Some(ui) = ui_weak_find.upgrade() else {
                            return;
                        };

                        let media_source = ui.global::<SlintMediaSource>();
                        media_source.set_filter_results(
                            slint_utils::rust_items_to_slint_model(items, true),
                        );

                        media_source.set_is_loading(false);
                    }).unwrap();
                }),
            });

            let media_source = ui.global::<SlintMediaSource>();
            media_source.set_is_loading(true);
            media_source.set_find_results(ModelRc::default());
            filter_tx.send(cmd).ok();
        }
    });

    let find_tx = media_source_tx.clone();
    slint_media_source.on_find({
        let ui = ui_slint_media_source_find.upgrade().unwrap();
        move |id| {
            let ui_weak_find = ui.as_weak().clone();
            let cmd = MediaSourceCommand::Find(FindCommand {
                id: id.to_string(),
                callback: Box::new(|item_option| {
                    slint::invoke_from_event_loop(move || {
                        let Some(ui) = ui_weak_find.upgrade() else {
                            return;
                        };

                        let media_source = ui.global::<SlintMediaSource>();
                        if let Some(item) = item_option {
                            media_source.set_find_results(
                                slint_utils::rust_items_to_slint_model(vec![item], true),
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
            find_tx.send(cmd).ok();
        }
    });



    let slint_audio_player = ui.global::<SlintAudioPlayer>();
    slint_audio_player.on_play_test({
        let tx = player_cmd_tx.clone();
        move || {
            tx.send(PlayerCommand::PlayTest()).unwrap();
        }
    });

    slint_audio_player.on_play_media({
        let tx = player_cmd_tx.clone();
        move |file_name: SharedString| {
            tx.send(PlayerCommand::PlayMedia(file_name.to_string()))
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


    let ui_handle_player = ui.as_weak();


    slint::spawn_local(async move {

        while let Some(event) = player_evt_rx.recv().await {
            if let Some(ui) = ui_handle_player.upgrade() {
                let inner = ui.global::<SlintAudioPlayer>();

                match event {
                    PlayerEvent::Status(item_id, status) => {
                        inner.set_current_item_id(item_id.to_shared_string());
                        inner.set_status(status.to_shared_string());
                    }

                    PlayerEvent::Stopped => {}

                    PlayerEvent::Position(item_id, position) => {
                        inner.set_current_item_id(item_id.to_shared_string());
                        inner.set_position_formatted(format_duration(position).to_shared_string());
                    }
                }
            } else {
                // UI was dropped; stop listening
                break;
            }
        }
    })
        .unwrap();


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