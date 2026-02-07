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


Button Layout:

----------------
 USB-C Power

  o -> 502 / A22
  o -> not working ?? A25
  o -> not working ?? A23
  o -> 504 / A24

 USB-C Audio
 ------------
 */


#[derive(Copy, Clone)]
pub enum GpioPin {
    // somehow these seem to be shifted
    A22 = 502,
    A23 = 503,
    A24 = 504,
    A25 = 505,
}

impl GpioPin {
    pub fn to_u64(&self) -> u64 {
        *self as u64
    }

}
