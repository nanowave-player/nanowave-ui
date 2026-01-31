slint::include_modules!();

use background::start_tokio_background_tasks;
use slint::{Model, ModelRc, SharedString, VecModel};
use std::iter;


mod background;
mod database_wrapper;
mod entity;
mod migrator;
mod file_utils;

fn main() -> Result<(), slint::PlatformError> {

    let base_path = "media/";
    start_tokio_background_tasks(base_path);

    let ui = MainWindow::new()?;
    let ui_weak = ui.as_weak();

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
    let ui_slint_media_source_filter = ui_weak.clone();
    // let filter_ws_tx = ws_tx.clone();
    slint_media_source.on_filter({
        let ui = ui_slint_media_source_filter.upgrade().unwrap();
        move |query| {
            let media_source = ui.global::<SlintMediaSource>();
            media_source.set_is_loading(true);
            media_source.set_filter_results(ModelRc::default());
            println!("on_filter query: {}",query);
            // backend_service_filter.media_source_filter(query.to_string());
            // let msg = r#"{"jsonrpc": "2.0", "method":"media_source_filter", "params": {"query":"2"}, "id": "1"}"#;
            // let _ = filter_ws_tx.send(msg.to_string());
        }
    });

    // let backend_service_find = backend_service.clone();
    let ui_slint_media_source_find = ui_weak.clone();
    // let find_ws_tx = ws_tx.clone();

    slint_media_source.on_find({
        let ui = ui_slint_media_source_find.upgrade().unwrap();
        move |id| {
            let media_source = ui.global::<SlintMediaSource>();
            media_source.set_is_loading(true);
            media_source.set_find_results(ModelRc::default());
            // backend_service_find.media_source_find(id.to_string());
            println!("on_find id: {}", id);
            // let msg = r#"{"jsonrpc": "2.0", "method":"media_source_find", "params": {"id":"2"}, "id": "1"}"#;
            // let _ = find_ws_tx.send(msg.to_string());
        }
    });

    ui.run()
}