use std::any::type_name;
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc::UnboundedSender;
use crate::background::display_controller::DisplayCommand;
use crate::background::scheduler::scheduler_task_trait::SchedulerTaskTrait;
use crate::background::scheduler::scheduler_task_type::SchedulerTaskType;


pub enum DisplayAutoShutdownTaskEvent {
    ResetTimer
}

#[derive(Debug)]
pub struct DisplayAutoShutdownTask {
    last_reset: SystemTime,
    timeout: Duration,
    display_tx: UnboundedSender<DisplayCommand>
}


impl DisplayAutoShutdownTask {
    pub fn id() -> String {
        type_name::<Self>().to_string()
    }
    pub fn new(display_tx: UnboundedSender<DisplayCommand>, auto_shutdown_after_ms:u64) -> Self {
        Self {
            last_reset: SystemTime::now(),
            timeout: Duration::from_millis(auto_shutdown_after_ms),
            display_tx,
        }
    }
}


impl SchedulerTaskTrait for DisplayAutoShutdownTask {
    fn id(&self) -> String {
        Self::id() // my_module::MyStruct
    }

    fn task_type(&self) -> SchedulerTaskType {
        SchedulerTaskType::Permanent
    }

    fn reset(&mut self) {
        self.last_reset = SystemTime::now();
    }

    fn should_execute(&self) -> bool {
        let result = SystemTime::now().duration_since(self.last_reset);
        match result {
            Ok(elapsed) => elapsed >= self.timeout,
            Err(_) => false,
        }
    }

    /// Do the shutdown work; return Ok(true) if it actually executed.
    fn execute(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        if !self.should_execute() {
            return Ok(false);
        }
        println!("Auto-shutting down display…");
        let _ = self.display_tx.send(DisplayCommand::TurnOff);
        self.reset();

        Ok(true)
    }
}

