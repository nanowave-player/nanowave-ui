use crate::input_event::{InputEvent, InputEventAction, InputEventButton, InputEventDevice};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use sysfs_gpio::{Direction, Pin};
use tokio::sync::mpsc::UnboundedSender;


struct GpioHandler {

}

impl GpioHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run(&mut self, pin_numbers: Vec<u64>, gpio_tx: Arc<UnboundedSender<InputEvent>>) {

        for pin_num in pin_numbers {
            let gpio_tx_clone = gpio_tx.clone();
            let input = Pin::new(pin_num);


            /*
            // GPIO Number = (Bank letter: A=0, B=1 P=15,) × 32 + Pin number

            # A22 -> 502: vol- (possibly 22 instead of 502 because A=0)
            devmem 0x03001050 b 0x03
            echo 502 > /sys/class/gpio/export

            # A23 -> 503: play
            devmem 0x0300105C b 0x03
            echo 503 > /sys/class/gpio/export

            # A24 -> 504: power
            devmem 0x03001060 b 0x03
            echo 504 > /sys/class/gpio/export

            # A25 -> 505: vol+
            devmem 0x03001054 b 0x03
            echo 505 > /sys/class/gpio/export
             */
            let input_event_button_opt = match pin_num {
                1 => {Some(InputEventButton::Power)}
                2 => {Some(InputEventButton::VolumeIncrease)}
                3 => {Some(InputEventButton::PlayPause)}
                4 => {Some(InputEventButton::VolumeDecrease)}
                _ => {None}
            };

            if input_event_button_opt.is_none() {
                println!("no button mapping for {}", pin_num);
                continue;
            }

            let _ = input.with_exported(|| {
                let input_event_button = input_event_button_opt.unwrap();

                input.set_direction(Direction::In)?;
                let mut prev_val: u8 = 255;
                loop {
                    let input_event_button = input_event_button.clone();
                    let val = input.get_value()?;
                    if val != prev_val {
                        println!("Pin State: {}", if val == 0 { "Low" } else { "High" });

                        let action = if val == 0 {
                            InputEventAction::Press
                        } else {
                            InputEventAction::Release
                        };

                        gpio_tx_clone.send(InputEvent::ButtonEvent(InputEventDevice::Gpio, input_event_button, action)).unwrap();

                        prev_val = val;
                    }
                    sleep(Duration::from_millis(10));
                }
            });
        }

    }
}