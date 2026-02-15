use crate::background::player::PlayerCommand;
use crate::input_event::InputEventAction::{Press, Release};
use crate::input_event::{InputEvent, InputEventButton};
use debounce::EventDebouncer;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use media_source::media_source_item::MediaSourceItem;
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
        prefs_tx: UnboundedSender<PreferencesCommand>,

    ) {
        println!("INPUT_HANDLER run");

        let delay = Duration::from_millis(250);

        let clicks = Arc::new(Mutex::new(0));
        let hold = Arc::new(Mutex::new(false));

        let clicks_clone = clicks.clone();
        let hold_clone = hold.clone();


        let player_tx_cancel_clone = player_tx.clone();

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

            let mut enable_touch_events = true;
            while let Some(event) = headset_rx.recv().await {
                match event {
                    InputEvent::ButtonEvent(_device, button, action) => {
                        match button {
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
                                    let _ = prefs_tx.send(PreferencesCommand::SetEnableTouchEvents(!enable_touch_events));
                                    enable_touch_events = !enable_touch_events;

                                    if enable_touch_events {
                                        self.turn_display_on();
                                    } else {
                                        self.turn_display_off();
                                    }

                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

    }

    fn turn_display_on(&self) {
        // Back to normal (may need to send LCD init cmds after re-enabling it):
        // devmem 0x30010a4 32 0x00

        // alternative: turn on backlight only
        // echo 1 > /sys/class/pwm/pwmchip8/pwm2/enable


        let address: u32 = 0x30010a4;
        let value: u32 = 0x00;
        peekpoke::write(address, value);
    }

    fn turn_display_off(&self) {
        // Reset/turn off LCD:
        // devmem 0x30010a4 32 0x04

        // alternative: turn off backlight only
        // echo 0 > /sys/class/pwm/pwmchip8/pwm2/enable
        let address: u32 = 0x30010a4;
        let value: u32 = 0x04;
        peekpoke::write(address, value);
    }

    /*
    fn display_brightness() {
        // min-value:0
        // max-value: Unknown, but should not be more than 2500
        // echo 1000 > /sys/class/pwm/pwmchip8/pwm2/duty_cycle
    }
    
     */
}
