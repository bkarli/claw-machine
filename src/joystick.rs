use core::cell::RefCell;
use core::future::poll_fn;
use arduino_hal::hal::port::Dynamic;
use arduino_hal::port::mode::{Input, PullUp};
use arduino_hal::port::Pin;
use avr_device::interrupt;

use crate::{Mutex, J_BACKWARD, J_FORWARD, J_LEFT, J_RIGHT};

/// An array of directions should be Async safe
/// If a joystick button is pressed in a direction the boolean value should be true => notify the
/// respective motor to move into a certain direction
static DIRECTIONS_ACTIVE: Mutex<RefCell<[JoystickDirection; 4]>> = Mutex::new(RefCell::new([JoystickDirection::BACKWARD(false), JoystickDirection::FORWARD(false), JoystickDirection::LEFT(false), JoystickDirection::RIGHT(false)]));


/**
Joystick Directions


*/
#[derive(Copy, Clone)]
pub enum JoystickDirection {
    LEFT(bool),
    RIGHT(bool),
    FORWARD(bool),
    BACKWARD(bool)
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

    pub async fn poll(&mut self) {
    }
}




/**
Pin Change interrupt triggered if a Joy Stick switch has been triggered
*/
#[avr_device::interrupt(atmega2560)]
#[allow(non_snake_case)]
fn PCINT0() {

    // We don't actually need to create a critical section as AVR suppresses other interrupts during
    // an interrupt
    interrupt::free(|cs| {
        let right_pin = J_RIGHT.borrow(cs).take().unwrap();
        let left_pin = J_LEFT.borrow(cs).take().unwrap();
        let forward_pin = J_FORWARD.borrow(cs).take().unwrap();
        let backward_pin = J_BACKWARD.borrow(cs).take().unwrap();
        let direction = DIRECTIONS_ACTIVE.borrow(cs).borrow_mut();

        // we don't want left and right movement at the same time
        // Physically shouldn't be possible with the joystick anyway but better safe than sorry
        if right_pin.is_low() {
            // notify the X-axis that it should move right
        } else if left_pin.is_low() {
            // notify the X-axis that it should move left
        }

        // again we don't want forward and backward movement at the same time
        if forward_pin.is_low() {
            // notify the Y-axis that it should move forward
        } else if backward_pin.is_low() {
            // notify the Y-axis that it should move backward
        }
    });

}

