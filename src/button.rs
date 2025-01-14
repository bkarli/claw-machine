//! This file holds the logic for the two UI buttons that the user can interact with
//! The logic is pretty simple as the UI buttons just advance the state of the system
//!
//! The ISR manages whether a button should be pressable or not



use avr_device::interrupt;
use crate::{B_END, B_START};
use crate::executor::wake_task;

/**
Pin Change interrupt triggered if a game button has been pressed
*/
#[avr_device::interrupt(atmega2560)]
#[allow(non_snake_case)]
fn PCINT1() {
    interrupt::free(|cs| {
        let end_btn = B_END.borrow(cs).take().unwrap();
        let start_btn = B_START.borrow(cs).take().unwrap();

        // check if the pin change interrupt was triggered by button press not release
        if end_btn.is_high() || start_btn.is_high() {
            // advance the state of the system by breaking the async executor loop
            wake_task(0xFFFF)
        }
    });
}