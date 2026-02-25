use std::time::{Duration, SystemTime};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use crate::background::display_controller::DisplayCommand;


pub enum SchedulerEvent {
    ResetTimer
}

pub struct Scheduler {
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
        }
    }

    pub fn reset_timer(&mut self) {
    }


    pub async fn run(
        &mut self,
        mut scheduler_rs: UnboundedReceiver<SchedulerEvent>,
        display_tx: UnboundedSender<DisplayCommand>
    ) {
        let mut display_off_reference_time = SystemTime::now();
        let display_off_after = Duration::from_secs(30);


        tokio::spawn(async move {
            loop {
                while let Some(event) = scheduler_rs.recv().await {
                    match event {
                        SchedulerEvent::ResetTimer => display_off_reference_time = SystemTime::now(),
                    }
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });

        loop {
            println!("Scheduler loop");
            let now = SystemTime::now();
            if now - display_off_after > display_off_reference_time {
                println!("scheduler: turn display off");
                let _ = display_tx.send(DisplayCommand::TurnOff);
                display_off_reference_time = SystemTime::now();
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    }
}