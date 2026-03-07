use crate::background::gpio_pin::GpioPin;
use crate::input_event::{InputEvent, InputEventAction, InputEventButton, InputEventDevice};
use std::sync::Arc;
use sysfs_gpio::{Direction, Edge, Pin};
use tokio::sync::mpsc::UnboundedSender;


use futures::future::join_all;
use futures::StreamExt;

pub struct GpioHandler {

}

impl GpioHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run(&mut self, gpio_pins: Vec<GpioPin>, gpio_tx: Arc<UnboundedSender<InputEvent>>) {
        println!("Starting GpioHandler");

        join_all(gpio_pins.into_iter().map(|p| {
            let gpio_tx = gpio_tx.clone();
            let pin = Pin::new(p.to_u64());
            tokio::task::spawn(monitor_pin(pin, gpio_tx))
        })).await;




        /*
        for gpio_pin in gpio_pins {
            let gpio_tx = gpio_tx.clone();
            // tokio::spawn(async move {
                let input = Pin::new(gpio_pin.to_u64());


                let input_event_button = match gpio_pin {
                    GpioPin::A22 => InputEventButton::VolumeDecrease,
                    GpioPin::A23 => InputEventButton::PlayPause,
                    GpioPin::A24 => InputEventButton::Power,
                    GpioPin::A25 => InputEventButton::VolumeIncrease,
                };

                println!("handling gpio pin {}", gpio_pin.to_u64());

                let _ = input.with_exported(|| {
                    input.set_direction(Direction::In)?;
                    let mut prev_val: u8 = 1;
                    loop {
                        let pin = gpio_pin.to_u64();
                        let input_event_button = input_event_button.clone();
                        let val = input.get_value()?;
                        if val != prev_val {
                            println!("gpio pin {} state: {} ({})", pin, if val == 0 { "Low" } else { "High" }, val);

                            let action = if val == 0 {
                                InputEventAction::Press
                            } else {
                                InputEventAction::Release
                            };

                            gpio_tx.send(InputEvent::ButtonEvent(InputEventDevice::Gpio, input_event_button, action)).unwrap();

                            prev_val = val;
                        }
                        sleep(Duration::from_millis(10));
                    }
                });
            // });

        }
        */
    }
}


async fn monitor_pin(pin: Pin, gpio_tx: Arc<UnboundedSender<InputEvent>>) -> Result<(), sysfs_gpio::Error> {
    pin.export()?;
    pin.set_direction(Direction::In)?;
    pin.set_edge(Edge::BothEdges)?;
    let mut gpio_events = pin.get_value_stream()?;
    while let Some(evt) = gpio_events.next().await {
        let val = evt?;
        println!("Pin {} changed value to {}", pin.get_pin_num(), val);

        let pin_num = pin.get_pin_num();

        let input_event_button = if pin_num == GpioPin::A22.to_u64() {
            InputEventButton::Power
        } else if pin_num == GpioPin::A23.to_u64() {
            InputEventButton::PlayPause
        } else if pin_num == GpioPin::A24.to_u64() {
            InputEventButton::VolumeDecrease
        } else if pin_num == GpioPin::A25.to_u64() {
            InputEventButton::VolumeIncrease
        } else {
            InputEventButton::PlayPause
        };

        let action = if val == 0 {
            InputEventAction::Press
        } else {
            InputEventAction::Release
        };

        gpio_tx.send(InputEvent::ButtonEvent(InputEventDevice::Gpio, input_event_button, action)).unwrap();
    }
    Ok(())
}
