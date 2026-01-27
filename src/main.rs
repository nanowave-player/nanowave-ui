slint::include_modules!();

use std::iter;
use crate::nanowave_service::NanoWaveService;
use slint::{Model, ModelRc, SharedString, VecModel};
use std::rc::Rc;
use tungstenite::{connect, Message};
use url::Url;
use crate::websockets::start_websocket_runtime;

mod nanowave_service;
mod websockets;

fn main() -> Result<(), slint::PlatformError> {
    /*
    // Change this to your actual WebSocket URL
    let url = Url::parse("ws://localhost:8080").unwrap();

    let (mut socket, _response) = connect(url.to_string()).expect("Failed to connect");

    let msg = r#"{"jsonrpc":"2.0","method":"media_source_filter","params":{"query":"2"},"id":"1"}"#;

    socket
        .write_message(Message::Text(msg.into()))
        .expect("Failed to send message");

    println!("Message sent");
    */

    let url_string = "ws://127.0.0.1:8080";
    let (ws_tx, mut ws_rx) = start_websocket_runtime(url_string.to_string());







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


    /*
    // UI → service command
    ui.on_play_requested(move |media_id: SharedString| {
        backend_service_clone.play_media(media_id.to_string());
    });
    */


    backend_service.on_message_received(move |msg| {
        // let ui = ui_slint_media_source_response.upgrade().unwrap();
        // let slint_media_source = ui.global::<SlintMediaSource>();


        println!("Received: {}", msg);


        /*
        while let Some(event) = source_evt_rx.recv().await {
            if let Some(ui) = ui_handle.upgrade() {
                let inner = ui.global::<SlintMediaSource>();

                match event {
                    MediaSourceEvent::FilterResults(items) => {
                        inner.set_filter_results(slint_helpers::utils::rust_items_to_slint_model(items, false));
                    }
                    MediaSourceEvent::FindResult(opt_item) => {
                        if let Some(item) = opt_item {
                            inner.set_find_results(slint_helpers::utils::rust_items_to_slint_model(vec![item], true));
                        } else {
                            // clear results if nothing found
                            inner.set_find_results(slint::ModelRc::default());
                        }
                    }
                }
            } else {
                // UI was dropped; stop listening
                break;
            }
        }
         */


        /*
        let timestamp = msg
            .get("params")
            .and_then(|p| p.get("timestamp"))
            .and_then(|t| t.as_str());

        if let (Some(ts), Some(ui)) = (timestamp, ui_backend_service.upgrade()) {
            ui.set_status(ts.into());
        }

         */
    });

    let slint_media_source = ui.global::<SlintMediaSource>();


    let ui_slint_media_source_filter = ui_weak.clone();
    let backend_service_filter = backend_service.clone();

    let filter_ws_tx = ws_tx.clone();
    slint_media_source.on_filter({
        let ui = ui_slint_media_source_filter.upgrade().unwrap();
        move |query| {
            let media_source = ui.global::<SlintMediaSource>();
            media_source.set_is_loading(true);
            media_source.set_filter_results(ModelRc::default());
            // backend_service_filter.media_source_filter(query.to_string());
            let msg = r#"{"jsonrpc": "2.0", "method":"media_source_filter", "params": {"query":"2"}, "id": "1"}"#;
            let _ = filter_ws_tx.send(msg.to_string());
        }
    });

    let ui_slint_media_source_find = ui_weak.clone();
    let backend_service_find = backend_service.clone();
    let find_ws_tx = ws_tx.clone();

    slint_media_source.on_find({
        let ui = ui_slint_media_source_find.upgrade().unwrap();
        move |id| {
            let media_source = ui.global::<SlintMediaSource>();
            media_source.set_is_loading(true);
            media_source.set_find_results(ModelRc::default());
            // backend_service_find.media_source_find(id.to_string());
            println!("find");
            let msg = r#"{"jsonrpc": "2.0", "method":"media_source_find", "params": {"id":"2"}, "id": "1"}"#;
            let _ = find_ws_tx.send(msg.to_string());
        }
    });


    std::thread::spawn(move || {
        while let Some(msg) = ws_rx.blocking_recv() {
            /*
            let ui = ui_handle.clone();
            slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui.upgrade() {
                    println!("Received from WS: {msg}");
                    // ui.set_something(msg.into());
                }
            })
                .unwrap();

             */
            println!("Received from WS: {msg}");
        }
    });

    // backend_service.run_in_background();
    ui.run()
}
