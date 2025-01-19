//! Main File for the claw machine
//!
//!

#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
#![feature(future_join)]

mod channel;
mod executor;
mod joystick;
mod stepper;
mod timer;

#[allow(unused_imports)]
use panic_halt as _;

use crate::timer::{GenericTicker, PrecisionTicker};
use arduino_hal::hal::port::Dynamic;
use arduino_hal::port::mode::{Input, Output, PullUp};
use arduino_hal::port::Pin;
use arduino_hal::simple_pwm::{IntoPwmPin};
use avr_device::interrupt;
use core::cell::{Cell, RefCell};
use core::pin::pin;
use crate::channel::{Channel, Receiver, Sender};
use crate::joystick::{JoystickDirection, JoystickSwitch};
use crate::stepper::{Stepper, StepperDirection};
use crate::stepper::StepperDirection::{ClockWise, CounterClockWise, Idle};

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

// Joy stick Pins
/// Joystick Right input Pin
static J_RIGHT: Mutex<Cell<Option<Pin<Input<PullUp>, Dynamic>>>> = Mutex::new(Cell::new(None));

/// Joystick Left input Pin
static J_LEFT: Mutex<Cell<Option<Pin<Input<PullUp>, Dynamic>>>> = Mutex::new(Cell::new(None));

/// Joystick Forward input Pin
static J_FORWARD: Mutex<Cell<Option<Pin<Input<PullUp>, Dynamic>>>> = Mutex::new(Cell::new(None));

/// Joystick Backward input Pin
static J_BACKWARD: Mutex<Cell<Option<Pin<Input<PullUp>, Dynamic>>>> = Mutex::new(Cell::new(None));


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

    let x_stepper_pulse = pins.d22.into_output().downgrade();

    let x_stepper_direction = pins.d23.into_output().downgrade();


    // even tough interrupts are not enabled yet still have to create critical section for mutex
    // set all static variables
    interrupt::free(|cs| {
        // set console
        *CONSOLE.borrow(cs).borrow_mut() = Some(serial);

        // set input pins
        J_RIGHT
            .borrow(cs)
            .set(Some(pins.d50.into_pull_up_input().downgrade()));
        J_LEFT
            .borrow(cs)
            .set(Some(pins.d51.into_pull_up_input().downgrade()));
        J_FORWARD
            .borrow(cs)
            .set(Some(pins.d52.into_pull_up_input().downgrade()));
        J_BACKWARD
            .borrow(cs)
            .set(Some(pins.d53.into_pull_up_input().downgrade()));

    });

    // enable interrupts for the device
    unsafe { interrupt::enable() };

    let test_stepper = pin!(test_async_stepper(x_stepper_direction, x_stepper_pulse));
    executor::run_task(&mut [test_stepper])
}

async fn test_async_stepper(
    direction_pin : Pin<Output, Dynamic>,
    pulse_pin: Pin<Output, Dynamic>
) {
    let mut motor = Stepper::new(direction_pin, pulse_pin, false);
    loop {
        for _ in 0..200 {
            motor.move_direction(ClockWise, 1000).await
        }

        for _ in 0..200 {
            motor.move_direction(CounterClockWise, 400).await
        }
    }
}
