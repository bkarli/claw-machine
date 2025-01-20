//! Main File for the claw machine
//!
//!

#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
#![feature(future_join)]


mod executor;

mod timer;

#[allow(unused_imports)]
use panic_halt as _;

use crate::timer::{delay_s, delay_us, GenericTicker, PrecisionTicker};

use arduino_hal::simple_pwm::{IntoPwmPin, Prescaler, Timer3Pwm};
use avr_device::interrupt;
use core::cell::{RefCell};
use core::pin::pin;

type Mutex<T> = interrupt::Mutex<T>;
type Console = arduino_hal::hal::usart::Usart0<arduino_hal::DefaultClock>;

/*
PIN Configuration:

Arduino Mega2560 rev3

INPUT:
    Joystick (PCINT0)
        1. Right: 50 PCINT3
        2. Left: 51 PCINT2
        3. Forward: 52 PCINT1
        4. Backward: 53 PCINT0

    Play Button (PCINT1)
        1. Start Button: 15 PCINT9
        2. End Button: 14 PCINT10

    Limit Switch (PCINT2)
        1. X Limit (left/right): A8 PCINT16
        2. Y Limit (forward/backward): A9 PCINT17
        3. Z Limit (Pulley up/down): A10 PCINT18

OUTPUT:
    Stepper Motor
        1. X-Pulse: 22
        2. X-Direction: 23

        3. Y-Pulse: 24
        4. Y-Direction: 25

        5. Y-Pulse-Inverted: 26
        6. Y-Direction-Inverted: 27

        7. Z-Pulse: 28
        8. Z-Direction: 29

    Servo Motor

        1. Claw: 5 (PWM)
*/




/// Create a console that can be used safely within an interrupt
static CONSOLE: Mutex<RefCell<Option<Console>>> = Mutex::new(RefCell::new(None));

/**
Entrypoint for the Program
*/
#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    PrecisionTicker::init(dp.TC0);
    GenericTicker::init(dp.TC1);

    // create a serial connection with the console output
    let serial = arduino_hal::default_serial!(dp, pins, 57600);

    unsafe { interrupt::enable() };

    interrupt::free(|cs| {
        *CONSOLE.borrow(cs).borrow_mut() = Some(serial);
    });


    let test_generic_timer = pin!(test_generic_timer());
    let test_precision_timer = pin!(test_precision_timer());
    executor::run_task(&mut [test_precision_timer, test_generic_timer]);
    // initialize static Tickers
}


async fn test_generic_timer() {

    delay_s(1).await;
    delay_s(10).await;

}


async fn test_precision_timer(){

    delay_us(1000).await;

    for i in 0..1000 {
        delay_us(1000).await;
    }
}
