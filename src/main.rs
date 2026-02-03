slint::include_modules!();

use crate::background::media_source::{FilterCommand, FindCommand, MediaSourceCommand};
use crate::config::Config;
use background::start_tokio_background_tasks;
use slint::{Model, ModelRc, SharedString, VecModel};
use std::iter;
use tokio::sync::mpsc;
use crate::background::player::PlayerEvent;

mod background;
mod database_wrapper;
mod entity;
mod migrator;
mod file_utils;
mod config;
mod slint_utils;

fn main() -> Result<(), slint::PlatformError> {
    let base_path = "media/";
    let (media_source_tx, media_source_rx) = tokio::sync::mpsc::unbounded_channel::<background::media_source::MediaSourceCommand>();
    let (player_cmd_tx, player_cmd_rx) = tokio::sync::mpsc::unbounded_channel::<background::player::PlayerCommand>();
    let (player_evt_tx, mut player_evt_rx) = mpsc::unbounded_channel::<PlayerEvent>();


    start_tokio_background_tasks(Config::new(base_path.to_string()),
                                 media_source_tx.clone(),
                                 media_source_rx,
                                 player_cmd_tx.clone(),
                                 player_cmd_rx,
                                 player_evt_tx.clone(),
                                 player_evt_rx
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
        // inner_ui.global::<SlintNavigation>().
        // inner_ui.global::<SlintNavigation>().set_history()

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


    // let backend_service_filter = backend_service.clone();
    let filter_tx = media_source_tx.clone();

    // let filter_ws_tx = ws_tx.clone();
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

        /*
        let ui = ui_slint_media_source_filter.upgrade().unwrap();
        move |query| {
            let media_source = ui.global::<SlintMediaSource>();
            media_source.set_is_loading(true);
            media_source.set_filter_results(ModelRc::default());
            println!("on_filter query: {}",query);

            let cmd = MediaSourceCommand::Filter(FilterCommand {
                query: query.to_string(),
                callback: Box::new(|items| {
                    println!("Found {} items", items.len());
                }),
            });

            filter_tx.send(cmd).ok();

        }

         */
    });

    // let backend_service_find = backend_service.clone();
    // let find_ws_tx = ws_tx.clone();
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





    ui.run()
}