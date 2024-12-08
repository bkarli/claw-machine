use arduino_hal::hal::port::Dynamic;
use arduino_hal::port::mode::{Input, PullUp};
use arduino_hal::port::Pin;

/**
Joystick Directions
*/
enum JoystickDirection {
    LEFT,
    RIGHT,
    FORWARD,
    BACKWARD
}

/**
struct for the joystick
*/
pub struct Joystick {
    left_pin: Pin<Input<PullUp>, Dynamic>,
    right_pin: Pin<Input<PullUp>, Dynamic>,
    forward_pin: Pin<Input<PullUp>, Dynamic>,
    backward_pin: Pin<Input<PullUp>, Dynamic>,
}


impl Joystick {
    pub fn new(
        left_pin: Pin<Input<PullUp>, Dynamic>,
        right_pin: Pin<Input<PullUp>, Dynamic>,
        forward_pin: Pin<Input<PullUp>, Dynamic>,
        backward_pin: Pin<Input<PullUp>, Dynamic>,
    ) -> Self {
        Self{
            left_pin,
            right_pin,
            forward_pin,
            backward_pin
        }
    }

    pub fn poll(&mut self) {

    }
}

