use core::future::Future;
use core::task::{Context, Poll};
use arduino_hal::hal::port::Dynamic;
use arduino_hal::port::mode::{Input, PullUp};
use arduino_hal::port::Pin;
use avr_device::interrupt;
use crate::B_END;
use crate::executor::ExtWaker;
use crate::timer::PrecisionTimer;

/**
Enum for All possible UI Buttons that a User can press

GameStart => Starts the game
GameFinish => Ends the game
*/
#[derive(Copy, Clone)]
pub enum ButtonType {
    GameStart,
    GameFinish
}

pub enum ButtonState {
    Idle,
    Debounce(PrecisionTimer),
}


/**
struct for the UI buttons
*/
pub struct Button {
    button_type: ButtonType,
    button_state: ButtonState,
    pin: Pin<Input<PullUp>, Dynamic>
}

impl Button {
    pub fn new(button_type: ButtonType, pin: Pin<Input<PullUp>, Dynamic>) -> Self {
        Self {
            button_type,
            button_state: ButtonState::Idle,
            pin
        }
    }
}



pub async fn button_task(button_type: ButtonType) {

}


/**
Pin Change interrupt triggered if a game button has been pressed
*/
#[avr_device::interrupt(atmega2560)]
#[allow(non_snake_case)]
fn PCINT1() {
    // We don't actually need to create a critical section as AVR suppresses other interrupts during
    // an Interrupt
    interrupt::free(|cs| {
        let end_btn = B_END.borrow(cs).take().unwrap();
        let start_btn = B_END.borrow(cs).take().unwrap();

        // HAL is checking Port directly should be fast enough so no PC are getting missed
        if (end_btn.is_low()) {
            // end btn has been pressed

        } else if (start_btn.is_low()) {
            // start btn has been pressed
            // theoretically we don't need to check the start btn again because it could have been
            // either the start btn or the end_btn but in the very unlikely event that an interrupt
            // has been triggered and the system reacts to slowly the btn press will just be ignored

        }
    });
}