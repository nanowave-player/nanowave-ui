slint::include_modules!();

use std::iter;
use crate::nanowave_service::NanoWaveService;
use slint::{Model, ModelRc, SharedString, VecModel};
use std::rc::Rc;

mod nanowave_service;

fn main() -> Result<(), slint::PlatformError> {
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









    let backend_service = Rc::new(NanoWaveService::new());

    let ui_backend_service = ui_weak.clone();
    backend_service.on_message_received(move |msg| {
        // let ui = ui_backend_service.upgrade().unwrap();
        println!("Received: {}", msg);

        let timestamp = msg
            .get("params")
            .and_then(|p| p.get("timestamp"))
            .and_then(|t| t.as_str());

        if let (Some(ts), Some(ui)) = (timestamp, ui_backend_service.upgrade()) {
            ui.set_status(ts.into());
        }
    });

    /*
    // UI → service command
    ui.on_play_requested(move |media_id: SharedString| {
        backend_service_clone.play_media(media_id.to_string());
    });
    */

    let backend_service_clone = backend_service.clone();
    let slint_media_source = ui.global::<SlintMediaSource>();
    let ui_slint_media_source = ui_weak.clone();
    slint_media_source.on_filter({
        let ui = ui_slint_media_source.upgrade().unwrap();

        let inner = ui.global::<SlintMediaSource>();
        inner.set_is_loading(true);
        inner.set_filter_results(ModelRc::default());

        move |query| {
            backend_service_clone.filter_media(query.to_string());
        }

        /*
        move |query| {
            filter_tx
                .send(MediaSourceCommand::Filter(query.to_string()))
                .unwrap();
        }

         */
    });
/* Todo
    let slint_media_source_find_ui = slint_app_window.clone_strong();
    let find_tx = source_cmd_tx.clone();
    slint_media_source.on_find({
        let inner = slint_media_source_find_ui.global::<SlintMediaSource>();
        inner.set_is_loading(true);
        inner.set_find_results(ModelRc::default());
        move |id| {
            find_tx
                .send(MediaSourceCommand::Find(id.to_string()))
                .unwrap();
        }
    });
*/
    backend_service.run_in_background();

    ui.run()
}
