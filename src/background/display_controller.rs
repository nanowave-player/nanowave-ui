use crate::background::scheduler::display_auto_shudown_task::DisplayAutoShutdownTask;
use crate::background::scheduler::scheduler::SchedulerEvent;
use std::time::Duration;
use tokio::fs;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

const DISPLAY_ON_OFF_FILE: &str = "/sys/class/pwm/pwmchip8/pwm2/enable";

pub enum DisplayCommand {
    TurnOn,
    TurnOff,
    Toggle,
    ChangeBrightness(f32),
}

pub struct DisplayController {}

impl DisplayController {
    pub fn new() -> DisplayController {
        Self {}
    }

    pub async fn is_display_on(&self) -> bool {
        let contents_result = fs::read_to_string(DISPLAY_ON_OFF_FILE).await;
        if let Ok(contents) = contents_result
            && contents.trim() == "0"
        {
            false
        } else {
            true
        }
    }

    pub async fn run(
        &mut self,
        scheduler_evt_tx: UnboundedSender<SchedulerEvent>,
        mut display_rx: UnboundedReceiver<DisplayCommand>,
    ) {
        loop {
            while let Some(event) = display_rx.recv().await {
                match event {
                    DisplayCommand::TurnOff => self.switch_display(false).await,
                    DisplayCommand::TurnOn => self.switch_display(true).await,
                    DisplayCommand::Toggle => self.toggle_display().await,
                    DisplayCommand::ChangeBrightness(brightness_perscent) => {
                        println!("Change brightness to {}", brightness_perscent);
                    }
                }
                let _ = scheduler_evt_tx.send(SchedulerEvent::Reset(DisplayAutoShutdownTask::id()));
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    async fn switch_display(&self, on: bool) {
        let value = if on { "1" } else { "0" };
        let _ = fs::write(DISPLAY_ON_OFF_FILE, value).await;
    }

    async fn toggle_display(&self) {
        self.switch_display(!self.is_display_on().await).await;
    }
}
