use crate::background::scheduler::scheduler::SchedulerEvent;
use evdev::{AbsoluteAxisCode, Device, KeyCode};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use crate::background::scheduler::display_auto_shudown_task::DisplayAutoShutdownTask;

pub struct TouchHandler {}


impl TouchHandler {
    pub fn new() -> Self {
        Self {}
    }

    fn is_touch_device(&self, device: &Device) -> bool {
        let required_axis_count = 2;
        let mt_axes = [
            AbsoluteAxisCode::ABS_MT_POSITION_X,
            AbsoluteAxisCode::ABS_MT_POSITION_Y,
            AbsoluteAxisCode::ABS_MT_TOUCH_MAJOR,
            AbsoluteAxisCode::ABS_MT_WIDTH_MAJOR,
        ];

        // Must have at least X/Y position + touch size
        let mut supported_axis_count = 0;
        for axis in mt_axes.iter() {
            if device
                .supported_absolute_axes()
                .map_or(false, |axes| axes.contains(*axis))
            {
                supported_axis_count += 1;
            }
        }

        println!("supported_axis_count: {}", supported_axis_count);

        // Touch devices typically have 3+ MT axes
        supported_axis_count >= required_axis_count
    }
    pub async fn run(
        &mut self,
        device_paths: Vec<&str>,
        scheduler_evt_tx: UnboundedSender<SchedulerEvent>,
    ) {
        let mut device_option: Option<Device> = None;

        loop {
            if let Some(device) = &mut device_option {
                let mut send_events: Vec<SchedulerEvent> = vec![];
                match device.fetch_events() {
                    Ok(events) => {
                        for event in events {
                            // println!("touch event {:?}", event.destructure())
                            let _ = scheduler_evt_tx.send(SchedulerEvent::Reset(DisplayAutoShutdownTask::id()));
                            break;
                        }
                    }
                    Err(e) => {
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                }

                for e in send_events {
                    scheduler_evt_tx.send(e).ok();
                }
            } else {
                println!("no touch device");

                for path_str in &device_paths {
                    let path = Path::new(path_str);
                    if !Path::exists(path) {
                        println!("path did not exists {}", path_str);
                        continue;
                    }
                    let device_result = Device::open(path_str);
                    if let Ok(d) = device_result {

                        if self.is_touch_device(&d) {
                            println!("valid touch device found");

                            // todo better handling
                            let _ = d.set_nonblocking(true);

                            device_option = Some(d);
                        } else {
                            println!("not a valid touch device");
                        }
                    } else {
                        println!("error opening touch device: {:?}", device_result);
                    }
                }

                tokio::time::sleep(Duration::from_secs(5)).await;
            }

            tokio::time::sleep(Duration::from_millis(500)).await;

        }
    }
}
