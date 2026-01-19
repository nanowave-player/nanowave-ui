slint::include_modules!();

use slint::SharedString;
use std::rc::Rc;
use std::thread;
use crate::nanowave_service::NanoWaveService;

mod nanowave_service;

fn main() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;
    let ui_weak = ui.as_weak();


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

    // Start background service

    slint::spawn_local(async move {
        service.run_in_background();
    }).unwrap();
//.expect("Failed to spawn NanoWaveService");


    ui.run()
}
