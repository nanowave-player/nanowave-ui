use crate::background::display_controller::DisplayCommand;
use crate::background::player::PlayerCommand;
use crate::input_event::InputEventAction::{Press, Release};
use crate::input_event::{InputEvent, InputEventButton};
use debounce::EventDebouncer;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};


#[derive(Debug)]
pub enum PreferencesCommand {
     SetEnableTouchEvents(bool),
}

pub struct InputHandler {
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
        }
    }

    pub async fn run(
        &mut self,
        mut headset_rx: UnboundedReceiver<InputEvent>,
        player_tx: Arc<UnboundedSender<PlayerCommand>>,
        display_tx:  UnboundedSender<DisplayCommand>,
    ) {
        println!("INPUT_HANDLER run");

        let delay = Duration::from_millis(250);

        let clicks = Arc::new(Mutex::new(0));
        let hold = Arc::new(Mutex::new(false));

        let clicks_clone = clicks.clone();
        let hold_clone = hold.clone();


        let player_tx_cancel_clone = player_tx.clone();
        let player_tx_volume_clone = player_tx.clone();



        let debouncer = EventDebouncer::new(delay, move |_str: &str| {
            let mut clicks_lock = clicks_clone.lock().unwrap();
            let hold_lock = hold_clone.lock().unwrap();

            println!("execute debouncer: clicks: {}, hold: {}", *clicks_lock, *hold_lock);
            // let _ = player_tx.clone().send(PlayerCommand::PlayPause);

            let player_tx_clone = player_tx.clone();

            player_tx_clone.send(PlayerCommand::CancelOngoing()).ok();

            if *hold_lock {
                match *clicks_lock {
                    1 => {
                        player_tx_clone.send(PlayerCommand::Rewind()).ok();
                    },
                    2 => {
                        player_tx_clone.send(PlayerCommand::FastForward()).ok();
                    },
                    3 => {
                        player_tx_clone.send(PlayerCommand::Rewind()).ok();
                    }
                    _ => {}
                }
            } else {
                match *clicks_lock {
                    1 => {
                        player_tx_clone.send(PlayerCommand::Toggle()).ok();
                    },
                    2 => {
                        player_tx_clone.send(PlayerCommand::Next()).ok();
                    },
                    3 => {
                        player_tx_clone.send(PlayerCommand::Previous()).ok();
                    }
                    _ => {}
                }
            }



            *clicks_lock = 0;
            drop(clicks_lock);
            drop(hold_lock);
        });


        let cancel_cb = || {
            let player_tx_clone = player_tx_cancel_clone.clone();
            player_tx_clone.send(PlayerCommand::CancelOngoing()).ok();
        };




        loop {
            while let Some(event) = headset_rx.recv().await {
                match event {
                    InputEvent::ButtonEvent(_device, button, action) => {
                        match button {
                            InputEventButton::VolumeIncrease => {
                                if action == Release {
                                    let _ = player_tx_volume_clone.send(PlayerCommand::IncreaseVolume);
                                }
                            }
                            InputEventButton::VolumeDecrease => {
                                if action == Release {
                                    let _ = player_tx_volume_clone.send(PlayerCommand::DecreaseVolume);
                                }
                            }
                            InputEventButton::PlayPause => {
                                let mut should_execute = true;
                                let mut clicks_lock = clicks.lock().unwrap();
                                let mut hold_lock = hold.lock().unwrap();

                                match action {
                                    Press => {
                                        *clicks_lock = *clicks_lock + 1;
                                        *hold_lock = true;
                                    }
                                    Release => {
                                        // do not execute debouncer if we have a release after a long hold
                                        should_execute = *clicks_lock > 0;
                                        *hold_lock = false;
                                    }
                                }

                                // println!("clicks: {}, hold: {}", *clicks_lock, *hold_lock);
                                drop(clicks_lock);
                                drop(hold_lock);
                                if should_execute {
                                    debouncer.put("");
                                } else {
                                    cancel_cb();
                                }
                            },
                            InputEventButton::Power => {
                                if action == Release {
                                    println!("Power button released");
                                    let _ = display_tx.send(DisplayCommand::Toggle);
                                }
                            }

                        }
                    }
                }
            }
        }
    }

}
