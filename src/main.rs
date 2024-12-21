//! Main File for the claw machine
//!
//!

#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod timer;
mod button;
mod limit_switch;
mod joystick;
mod game;
mod executor;
mod stepper;

use panic_halt as _;

use core::cell::{Cell, Ref, RefCell};
use arduino_hal::hal::port::{PB0, PB1, PB2, PB3, PJ0, PJ1, PK0, PK1, PK2};
use arduino_hal::pins;
use arduino_hal::port::mode::{Input, PullUp};
use arduino_hal::port::Pin;
use crate::timer::{GenericTicker, PrecisionTicker};
use avr_device::interrupt;

type Mutex<T> = interrupt::Mutex<T>;
type Console = arduino_hal::hal::usart::Usart0<arduino_hal::DefaultClock>;


/*
PIN Configuration:

Arduino Mega2560 rev3

INPUT:
    Joystick (PCINT0)
        1. Right: 50
        2. Left: 51
        3. Forward: 52
        4. Backward: 53

    Play Button (PCINT1)
        1. Start Button: 15
        2. End Button: 14

    Limit Switch (PCINT2)
        1. X Limit (left/right): A8
        2. Y Limit (forward/backward): A9
        3. Z Limit (Pulley up/down): A10

OUTPUT:
    Stepper Motor
        1. X-Pulse: 22
        2. X-Direction: 23

        3. Y-Pulse: 24
        4. Y-Direction: 25

        5. Z-Pulse: 26
        6. Z-Direction: 27

    Servo Motor
        1. Claw: 13 (PWM)
*/

// Joy stick Pins
/// Joystick Right input Pin
static J_RIGHT: Mutex<Cell<Option<Pin<Input<PullUp>, PB3>>>> = Mutex::new(Cell::new(None));

/// Joystick Left input Pin
static J_LEFT: Mutex<Cell<Option<Pin<Input<PullUp>, PB2>>>> = Mutex::new(Cell::new(None));

/// Joystick Forward input Pin
static J_FORWARD: Mutex<Cell<Option<Pin<Input<PullUp>, PB1>>>> = Mutex::new(Cell::new(None));

/// Joystick Backward input Pin
static J_BACKWARD: Mutex<Cell<Option<Pin<Input<PullUp>, PB0>>>> = Mutex::new(Cell::new(None));

// Button Pins
/// UI Button start input Pin
static B_START: Mutex<Cell<Option<Pin<Input<PullUp>, PJ0>>>> = Mutex::new(Cell::new(None));

/// UI Button end input Pin
static B_END: Mutex<Cell<Option<Pin<Input<PullUp>, PJ1>>>> = Mutex::new(Cell::new(None));

// Limit switch Pins
/// Limit switch X
static X_LIMIT: Mutex<Cell<Option<Pin<Input<PullUp>, PK0>>>> = Mutex::new(Cell::new(None));

/// Limit switch Y
static Y_LIMIT: Mutex<Cell<Option<Pin<Input<PullUp>, PK1>>>> = Mutex::new(Cell::new(None));

/// Limit switch Z
static Z_LIMIT: Mutex<Cell<Option<Pin<Input<PullUp>, PK2>>>> = Mutex::new(Cell::new(None));

/// Create a console that can be used safely within an interrupt
static CONSOLE: Mutex<RefCell<Option<Console>>> = Mutex::new(RefCell::new(None));



/**
Entrypoint for the Program
*/
#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // initialize static Tickers
    PrecisionTicker::init(dp.TC0);
    GenericTicker::init(dp.TC1);


    // create a serial connection with the console output
    let serial = arduino_hal::default_serial!(dp, pins, 57600);


    // even tough interrupts are not enabled yet still have to create critical section for mutex
    // set all static variables
    interrupt::free(|cs| {
        // set console
        *CONSOLE.borrow(cs).borrow_mut() = Some(serial);

        // set input pins
        J_RIGHT.borrow(cs).set(Some(pins.d50.into_pull_up_input()));
        J_LEFT.borrow(cs).set(Some(pins.d51.into_pull_up_input()));
        J_FORWARD.borrow(cs).set(Some(pins.d52.into_pull_up_input()));
        J_BACKWARD.borrow(cs).set(Some(pins.d53.into_pull_up_input()));

        B_START.borrow(cs).set(Some(pins.d15.into_pull_up_input()));
        B_END.borrow(cs).set(Some(pins.d14.into_pull_up_input()));

        X_LIMIT.borrow(cs).set(Some(pins.a8.into_pull_up_input()));
        Y_LIMIT.borrow(cs).set(Some(pins.a9.into_pull_up_input()));

        // TODO: set output pins

    });

    // enable interrupts for the device
    unsafe { interrupt::enable() };

    loop {
        avr_device::asm::sleep()
    }
}
