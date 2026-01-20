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







    let service = Rc::new(NanoWaveService::new());
    let service_clone = service.clone();

    service.on_message_received(move |msg| {
        println!("Received: {}", msg);

        let timestamp = msg
            .get("params")
            .and_then(|p| p.get("timestamp"))
            .and_then(|t| t.as_str());

        if let (Some(ts), Some(ui)) = (timestamp, ui_weak.upgrade()) {
            ui.set_status(ts.into());
        }
    });

    // UI → service command
    ui.on_play_requested(move |media_id: SharedString| {
        service_clone.play_media(media_id.to_string());
    });

    service.run_in_background();

    ui.run()
}
