#![no_std]
#![no_main]

mod button;
mod servo_motor;

mod stepper_motor;

use panic_halt as _;


/**
PINS:


D2 Stepper Motor X
D3 Stepper Motor Y
D4 Stepper Motor Z

D5 X Direction Kill switch
D6 Y Direction Kill switch
D7 Z Direction Kill switch

D8 Joystick Positive X
D9 Joystick Negative X

D10 Joystick Positive Y
D11 Joystick Negative Z

D12 Start Button
D13 Grab Button

D14 Grab Servo Motor
*/
#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);


    loop {

    }
}
