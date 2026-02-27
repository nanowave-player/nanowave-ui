use crate::background::scheduler::scheduler_task_type::SchedulerTaskType;


pub trait SchedulerTaskTrait {
    fn id(&self) -> String;
    fn task_type(&self) -> SchedulerTaskType;
    fn reset(&mut self);
    fn should_execute(&self) -> bool;
    fn execute(&mut self) -> Result<bool, Box<dyn std::error::Error>>;
}
