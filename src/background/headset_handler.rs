use crate::input_event::InputEventButton::{PlayPause, VolumeDecrease, VolumeIncrease};
use crate::input_event::InputEventDevice::Headset;
use crate::input_event::{InputEvent, InputEventAction};
use evdev::{Device, EventSummary, KeyCode};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tracing::debug;

pub struct HeadsetHandler {}

impl HeadsetHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run(&mut self, device_paths: Vec<&str>, headset_tx: Arc<UnboundedSender<InputEvent>>) {
        let mut device_option: Option<Device> = None;

        /*
        loop {
            tokio::time::sleep(Duration::from_secs(2)).await;
            headset_tx.send(InputEvent::PlayPause).ok();
            debug!("sending InputEvent::PlayPause");
        }
        */
        /*
        LLM Sample
        let path_str = "/dev/input/event13";
        let mut device = Device::open(path_str).unwrap();
        let (headset_tx, headset_rx) = mpsc::unbounded_channel::<input_event::InputEvent>();

        for event in device.fetch_events().unwrap() {
            debug!("sending InputEvent::PlayPause");
            headset_tx.send(InputEvent::PlayPause).ok();
        }

         */

        loop {
            if let Some(device) = &mut device_option {
                let mut send_events : Vec<InputEvent> = vec![];
                match device.fetch_events() {
                    Ok(events) => {
                        for event in events {
                            let send_event_option = match event.destructure() {
                                EventSummary::Key(ev, KeyCode::KEY_PLAYPAUSE, 1) => {
                                    // let _ = player_button_cmd_tx.send(HandleButton(ButtonKey::PlayPause, ButtonAction::Press, ev.timestamp()));
                                    // let _ = evt_tx.send(PlayerEvent::ExternalTrigger(TriggerAction::Toggle));
                                    // debug!("PLAYPAUSE PRESSED: {:?}", ev);
                                    debug!("PLAYPAUSE PRESSED: {:?}", ev);
                                    Some(InputEvent::ButtonEvent(Headset, PlayPause, InputEventAction::Press))
                                    // Some(InputEvent::PlayPause)
                                }
                                EventSummary::Key(ev, KeyCode::KEY_PLAYPAUSE, 0) => {
                                    // let _ = player_button_cmd_tx.send(HandleButton(ButtonKey::PlayPause, ButtonAction::Release, ev.timestamp()));
                                    // let _ = evt_tx.send(PlayerEvent::ExternalTrigger(ButtonKey::PlayPause, ButtonAction::Release, ev.timestamp()));
                                    // debug!("PLAYPAUSE RELEASED: {:?}", ev);
                                    debug!("PLAYPAUSE RELEASED: {:?}", ev);
                                    Some(InputEvent::ButtonEvent(Headset, PlayPause, InputEventAction::Release))

                                }
                                EventSummary::Key(ev, KeyCode::KEY_VOLUMEUP, 1) => {
                                    debug!("VOLUME_UP PRESSED: {:?}", ev);

                                    Some(InputEvent::ButtonEvent(Headset, VolumeIncrease, InputEventAction::Press))

                                    //let _ = player_button_cmd_tx.send(HandleButton(ButtonKey::VolumeUp, ButtonAction::Press, ev.timestamp()));
                                }
                                EventSummary::Key(ev, KeyCode::KEY_VOLUMEUP, 0) => {
                                    //let _ = player_button_cmd_tx.send(HandleButton(ButtonKey::VolumeUp, ButtonAction::Release, ev.timestamp()));
                                    debug!("VOLUME_UP RELEASED: {:?}", ev);
                                    Some(InputEvent::ButtonEvent(Headset, VolumeIncrease, InputEventAction::Release))

                                }
                                EventSummary::Key(ev, KeyCode::KEY_VOLUMEDOWN, 1) => {
                                    //let _ = player_button_cmd_tx.send(HandleButton(ButtonKey::VolumeDown, ButtonAction::Press, ev.timestamp()));
                                    debug!("VOLUME_DOWN PRESSED: {:?}", ev);
                                    Some(InputEvent::ButtonEvent(Headset, VolumeDecrease, InputEventAction::Press))

                                }
                                EventSummary::Key(ev, KeyCode::KEY_VOLUMEDOWN, 0) => {
                                    //let _ = player_button_cmd_tx.send(HandleButton(ButtonKey::VolumeDown, ButtonAction::Release, ev.timestamp()));
                                    debug!("VOLUME_DOWN RELEASED: {:?}", ev);
                                    Some(InputEvent::ButtonEvent(Headset, VolumeDecrease, InputEventAction::Release))
                                }
                                _ => {
                                    None
                                }
                            };

                            if let Some(send_event) = send_event_option {
                                send_events.push(send_event);
                            }

                        }
                    }
                    /*
                    Err(WouldBlock) => {
                        // No events available - continue polling
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }

                     */


                    Err(_e) => {
                        /*
                        // ignore errors for now
                        if e.kind() == std::io::ErrorKind::WouldBlock {
                            // headset_tx.send(InputEvent::PlayPause).ok();

                        } else {
                            edebug!("Error: {:?}", e);
                        }

                         */
                        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

                    },
                }

                for e in send_events {
                    headset_tx.send(e).ok();
                }

                 /*
                 for event in device.fetch_events().unwrap() {
                    debug!("sending InputEvent::PlayPause");
                    headset_tx.send(InputEvent::PlayPause).ok();
                    continue;
                    let tx_input_event_option = match event.destructure() {
                        EventSummary::Key(ev, KeyCode::KEY_PLAYPAUSE, 1) => {
                            // let _ = player_button_cmd_tx.send(HandleButton(ButtonKey::PlayPause, ButtonAction::Press, ev.timestamp()));
                            // let _ = evt_tx.send(PlayerEvent::ExternalTrigger(TriggerAction::Toggle));
                            // debug!("PLAYPAUSE PRESSED: {:?}", ev);
                            debug!("PLAYPAUSE PRESSED: {:?}", ev);
                            // Some(InputEvent::ButtonEvent(Headset, PlayPause, InputEventAction::Press))
                            Some(InputEvent::PlayPause)
                        }
                        EventSummary::Key(ev, KeyCode::KEY_PLAYPAUSE, 0) => {
                            // let _ = player_button_cmd_tx.send(HandleButton(ButtonKey::PlayPause, ButtonAction::Release, ev.timestamp()));
                            // let _ = evt_tx.send(PlayerEvent::ExternalTrigger(ButtonKey::PlayPause, ButtonAction::Release, ev.timestamp()));
                            // debug!("PLAYPAUSE RELEASED: {:?}", ev);
                            debug!("PLAYPAUSE RELEASED: {:?}", ev);
                            Some(InputEvent::ButtonEvent(Headset, PlayPause, InputEventAction::Release))

                        }
                        EventSummary::Key(ev, KeyCode::KEY_VOLUMEUP, 1) => {
                            debug!("VOLUME_UP PRESSED: {:?}", ev);

                            Some(InputEvent::ButtonEvent(Headset, VolumeIncrease, InputEventAction::Press))

                            //let _ = player_button_cmd_tx.send(HandleButton(ButtonKey::VolumeUp, ButtonAction::Press, ev.timestamp()));
                        }
                        EventSummary::Key(ev, KeyCode::KEY_VOLUMEUP, 0) => {
                            //let _ = player_button_cmd_tx.send(HandleButton(ButtonKey::VolumeUp, ButtonAction::Release, ev.timestamp()));
                            debug!("VOLUME_UP RELEASED: {:?}", ev);
                            Some(InputEvent::ButtonEvent(Headset, VolumeIncrease, InputEventAction::Release))

                        }
                        EventSummary::Key(ev, KeyCode::KEY_VOLUMEDOWN, 1) => {
                            //let _ = player_button_cmd_tx.send(HandleButton(ButtonKey::VolumeDown, ButtonAction::Press, ev.timestamp()));
                            debug!("VOLUME_DOWN PRESSED: {:?}", ev);
                            Some(InputEvent::ButtonEvent(Headset, VolumeDecrease, InputEventAction::Press))

                        }
                        EventSummary::Key(ev, KeyCode::KEY_VOLUMEDOWN, 0) => {
                            //let _ = player_button_cmd_tx.send(HandleButton(ButtonKey::VolumeDown, ButtonAction::Release, ev.timestamp()));
                            debug!("VOLUME_DOWN RELEASED: {:?}", ev);
                            Some(InputEvent::ButtonEvent(Headset, VolumeDecrease, InputEventAction::Release))
                        }
                        _ => {
                            None
                        }
                    };

                    if let Some(tx_input_event) = tx_input_event_option {
                        let send_result = headset_tx.send(tx_input_event);
                        debug!("SEND: {:?}", send_result);
                    }
                }

                  */
            } else {
                for path_str in &device_paths {
                    let path = Path::new(path_str);
                    if !Path::exists(path) {
                        continue;
                    }
                    let device_result = Device::open(path_str);
                    if let Ok(d) = device_result {


                        if self.is_headset_remote(&d) {
                            // todo better handling
                            let _non = d.set_nonblocking(true);

                            device_option = Some(d);
                            /*
                            let debug_device_result = Device::open(path_str);
                            if let Ok(d2) = debug_device_result {

                                let to_string = dbg!(d2.to_string());
                                let name = dbg!(d2.name());
                                let absinfo = d2.get_absinfo();
                                let properties = dbg!(d2.properties());
                                let misc_properties = dbg!(d2.misc_properties());
                                let supported_events = dbg!(d2.supported_events());
                                let supported_keys = dbg!(d2.supported_keys());
                                let supported_sounds = dbg!(d2.supported_sounds());
                                let x = "";
                            }
                             */

                        }
                    }
                }
            }
        }
    }

    fn is_headset_remote(&self, device: &Device) -> bool {
        if let Some(supported_keys) = device.supported_keys() {
            // KEY_VOICECOMMAND seems to be supported by all USB-C to Audio adapters
            // todo: otherwise it would be possible to add some configurable device name
            return supported_keys.contains(KeyCode::KEY_VOICECOMMAND);
        }
        false
    }
}
