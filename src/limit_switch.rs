use avr_device::interrupt;
use crate::{X_LIMIT, Y_LIMIT, Z_LIMIT};

/**
struct for the limit switches
*/
struct LimitSwitch {}

/**
Pin Change interrupt triggered if a limit switch has been triggered
*/
#[avr_device::interrupt(atmega2560)]
#[allow(non_snake_case)]
fn PCINT2() {
    // We don't actually need to create a critical section as AVR suppresses other interrupts during
    // an Interrupt
    interrupt::free(|cs| {
        let limit_x_pin = X_LIMIT.borrow(cs).take().unwrap();
        let limit_y_pin = Y_LIMIT.borrow(cs).take().unwrap();
        let limit_z_pin = Z_LIMIT.borrow(cs).take().unwrap();

        // check which limit switch has triggered the interrupt
        if limit_x_pin.is_low() {
            // notify the X-axis
        } else if limit_y_pin.is_low() {
            // notify the Y-axis
        } else if limit_z_pin.is_low() {
            // notify the Z-axis
        }
    } );
}