pub enum InputEventDevice {
    Headset,
    Gpio
}

pub enum InputEventButton {
    Power,
    VolumeIncrease,
    VolumeDecrease,
    PlayPause,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEventAction {
    Press,
    Release,
}

pub enum InputEvent {
    ButtonEvent(InputEventDevice, InputEventButton, InputEventAction),
    PlayPause
}

