use crate::background::player::PlayerCommand;
use crate::input_event::InputEventAction::{Press, Release};
use crate::input_event::InputEvent;
use debounce::EventDebouncer;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

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
    ) {
        println!("INPUT_HANDLER run");

        let delay = Duration::from_millis(250);

        let clicks = Arc::new(Mutex::new(0));
        let hold = Arc::new(Mutex::new(false));

        let clicks_clone = clicks.clone();
        let hold_clone = hold.clone();


        let debouncer = EventDebouncer::new(delay, move |str: &str| {
            let mut clicks_lock = clicks_clone.lock().unwrap();
            let hold_lock = hold_clone.lock().unwrap();

            println!("execute debouncer: clicks: {}, hold: {}", *clicks_lock, *hold_lock);
            // let _ = player_tx.clone().send(PlayerCommand::PlayPause);

            let player_tx_clone = player_tx.clone();

            if *hold_lock {

            } else {
                match *clicks_lock {
                    1 => {
                        player_tx_clone.send(PlayerCommand::Toggle()).ok();
                    },
                    _ => {}
                }
            }



            *clicks_lock = 0;
            drop(clicks_lock);
            drop(hold_lock);
        });


        loop {

            while let Some(event) = headset_rx.recv().await {
                match event {
                    InputEvent::ButtonEvent(device, button, action) => {

                        // todo: handle different devices and buttons

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
                        }
                    }
                    _ => {}
                }
            }
        }

    }


}
