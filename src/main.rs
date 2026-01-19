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

            if let Some(ui) = ui_weak.upgrade() {
                // Example: update UI safely
                ui.set_status(format!("Message: {}", msg).into());
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
