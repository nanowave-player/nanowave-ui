use evdev::{Device, EventSummary, KeyCode};
use std::path::Path;
use tokio::sync::mpsc::UnboundedSender;
use crate::input_event::InputEvent;
/*
let handle = thread::spawn(move || {
        loop {
            let device_paths = vec!["/dev/input/event1", "/dev/input/event13"];

            let mut device_opt: Option<Device> = None;
            for path_str in device_paths {
                let path = Path::new(path_str);
                if !Path::exists(path) {
                    continue;
                }
                let device_result = Device::open(path_str);
                if device_result.is_err() {
                    continue;
                }

                let d = device_result.unwrap();
                if d.name().is_some() && d.name().unwrap().contains("Apple") {
                    device_opt = Some(d);
                }

            }


            if device_opt.is_none() {
                thread::sleep(Duration::from_millis(5000));
                continue;
            }

            let mut device = device_opt.unwrap();

 */

pub struct HeadsetHandler {}

// InputEvent
// Source: Headset
// Key: PlayPause, VolUp, VolDown, Power
// Direction: Down / Up
//



impl HeadsetHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run(
        &mut self,
        device_paths: Vec<&str>, headset_tx: UnboundedSender<InputEvent>
    )
    {
        let mut device_option: Option<Device> = None;
        loop {
            for path_str in &device_paths {
                let path = Path::new(path_str);
                if !Path::exists(path) {
                    continue;
                }
                let device_result = Device::open(path_str);
                if device_result.is_err() {
                    continue;
                }

                let d = device_result.unwrap();
                // todo: this is limiting everything to apple devices

                if d.name().is_some() && d.name().unwrap().contains("Apple") {
                    println!("props: {:?}", d.properties());
                    device_option = Some(d);
                }

            }


            if let Some(device) = &mut device_option {
                for event in device.fetch_events().unwrap() {
                    // let _ = evt_tx.send(ev);

                    let mut trigger_action = false;
                    let mut event_str = "";
                    match event.destructure() {
                        EventSummary::Key(ev, KeyCode::KEY_PLAYPAUSE, 1) => {
                            // let _ = player_button_cmd_tx.send(HandleButton(ButtonKey::PlayPause, ButtonAction::Press, ev.timestamp()));
                            // let _ = evt_tx.send(PlayerEvent::ExternalTrigger(TriggerAction::Toggle));
                            // println!("PLAYPAUSE PRESSED: {:?}", ev);
                            event_str = "PLAYPAUSE (PRESS)  ";
                        }
                        EventSummary::Key(ev, KeyCode::KEY_PLAYPAUSE, 0) => {
                            // let _ = player_button_cmd_tx.send(HandleButton(ButtonKey::PlayPause, ButtonAction::Release, ev.timestamp()));
                            // let _ = evt_tx.send(PlayerEvent::ExternalTrigger(ButtonKey::PlayPause, ButtonAction::Release, ev.timestamp()));
                            // println!("PLAYPAUSE RELEASED: {:?}", ev);
                            event_str = "PLAYPAUSE (RELEASE)";
                        }
                        EventSummary::Key(ev, KeyCode::KEY_VOLUMEUP, 1) => {
                            println!("VOLUME_UP PRESSED: {:?}", ev);
                            //let _ = player_button_cmd_tx.send(HandleButton(ButtonKey::VolumeUp, ButtonAction::Press, ev.timestamp()));
                        }
                        EventSummary::Key(ev, KeyCode::KEY_VOLUMEUP, 0) => {
                            //let _ = player_button_cmd_tx.send(HandleButton(ButtonKey::VolumeUp, ButtonAction::Release, ev.timestamp()));
                            println!("VOLUME_UP RELEASED: {:?}", ev);
                        }
                        EventSummary::Key(ev, KeyCode::KEY_VOLUMEDOWN, 1) => {
                            //let _ = player_button_cmd_tx.send(HandleButton(ButtonKey::VolumeDown, ButtonAction::Press, ev.timestamp()));
                            println!("VOLUME_DOWN PRESSED: {:?}", ev);
                        }
                        EventSummary::Key(ev, KeyCode::KEY_VOLUMEDOWN, 0) => {
                            //let _ = player_button_cmd_tx.send(HandleButton(ButtonKey::VolumeDown, ButtonAction::Release, ev.timestamp()));
                            println!("VOLUME_DOWN RELEASED: {:?}", ev);
                        }
                        _ => {
                            // println!("got a different event: {:?}", event.destructure())
                        }
                    }
                    // let evt_tx_clone = evt_tx.clone();

                    // if trigger_action {}
                }
            }
        }
    }
}
