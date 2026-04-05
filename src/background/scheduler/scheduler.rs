use crate::background::scheduler::scheduler_task_trait::SchedulerTaskTrait;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedReceiver;

pub enum SchedulerEvent {
    Reset(String),
}

pub struct Scheduler {
    tasks: Vec<Box<dyn SchedulerTaskTrait + Send + Sync>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
        }
    }

    pub fn add_task(&mut self, task: Box<dyn SchedulerTaskTrait + Send + Sync>) -> &mut Self {
        self.tasks.push(task);
        self
    }


    pub async fn run(&mut self, mut scheduler_rx: UnboundedReceiver<SchedulerEvent>) {
        loop {
            let tasks = self.tasks.iter_mut(); // Clone Arc

            tokio::select! {
                Some(event) = scheduler_rx.recv() => {
                    match event {
                        SchedulerEvent::Reset(task_id) => {
                            for task in tasks {
                                if task.id() == task_id {
                                    task.reset();
                                }
                            }
                        }
                    }
                }
                _ = tokio::time::sleep(Duration::from_millis(100)) => {}
            }

            let tasks = self.tasks.iter_mut(); // Clone Arc

            for task in tasks {
                if task.should_execute() {
                    let _ = task.execute();
                    // todo: if task.task_type() == SchedulerTaskType::Permanent
                }
            }
        }


        /*
        tokio::spawn(async move {
            loop {
                while let Some(event) = scheduler_rx.recv().await {
                    match event {
                        SchedulerEvent::ResetTimer(task_id) => {
                            for task in &mut self.tasks {
                                if task.id() == task_id {
                                    task.reset();
                                }
                            }
                        },
                    }
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });
        */

        /*
        let tasks = Arc::new(self.tasks.iter().clone());
        tokio::spawn(async move {
            loop {
                while let Some(event) = scheduler_rx.recv().await {
                    match event {
                        SchedulerEvent::ResetTimer(task_id) => {
                            for task in tasks {
                                if task.id() == task_id {
                                    task.reset();
                                }
                            }
                        },
                    }
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });
        */

        /*
        loop {
            let mut tasks = self.tasks_arc().lock().await;
            for task in tasks.iter_mut() {
                if task.should_execute() {
                    task.execute();
                    // todo: if task.task_type() == SchedulerTaskType::Permanent
                }
            }
        }

         */

        /*
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
            debug!("Scheduler loop");
            let now = SystemTime::now();
            if now - display_off_after > display_off_reference_time {
                debug!("scheduler: turn display off");
                let _ = display_tx.send(DisplayCommand::TurnOff);
                display_off_reference_time = SystemTime::now();
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }

         */
    }
}
