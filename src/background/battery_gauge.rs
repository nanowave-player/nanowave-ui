use linux_embedded_hal::I2cdev;
use max170xx::Max17043;
use tokio::sync::mpsc::UnboundedSender;

pub enum StatusEvent {
    UpdateBattery(f32)
}
pub struct BatteryGauge {

}

impl BatteryGauge {

    pub fn new() -> Self {
        Self {}
    }


    pub async fn run(&mut self, status_tx: UnboundedSender<StatusEvent>) {
        // https://docs.rs/max170xx/latest/max170xx/
        loop {
            let dev_result = I2cdev::new("/dev/i2c-5");
            if let Ok(dev) = dev_result {

                let mut sensor = Max17043::new(dev);

                let soc_result = sensor.soc();
                if let Ok(soc) = soc_result {
                    let _ = status_tx.send(StatusEvent::UpdateBattery(soc));
                    println!("Charge: {:.2}%", soc);
                } else {
                    println!("Failed to soc: {:?}", soc_result);
                }

            } else {
                println!("Failed to i2c-5: {:?}", dev_result.err());
            }

            tokio::time::sleep(std::time::Duration::from_secs(120)).await;
        }
    }
}