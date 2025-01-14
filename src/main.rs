//! Main File for the claw machine
//!
//!

#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
#![feature(future_join)]

mod button;
mod channel;
mod executor;
mod game;
mod joystick;
mod limit_switch;
mod stepper;
mod timer;

#[allow(unused_imports)]
use panic_halt as _;

use crate::game::Game;
use crate::timer::{GenericTicker, PrecisionTicker};
use arduino_hal::hal::port::Dynamic;
use arduino_hal::port::mode::{Input, PullUp};
use arduino_hal::port::Pin;
use arduino_hal::simple_pwm::Prescaler::Prescale64;
use arduino_hal::simple_pwm::{IntoPwmPin, Timer3Pwm};
use avr_device::interrupt;
use core::cell::{Cell, RefCell};

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

// Button Pins
/// UI Button start input Pin
static B_START: Mutex<Cell<Option<Pin<Input<PullUp>, Dynamic>>>> = Mutex::new(Cell::new(None));

/// UI Button end input Pin
static B_END: Mutex<Cell<Option<Pin<Input<PullUp>, Dynamic>>>> = Mutex::new(Cell::new(None));

// Limit switch Pins
/// Limit switch X
static X_LIMIT: Mutex<Cell<Option<Pin<Input<PullUp>, Dynamic>>>> = Mutex::new(Cell::new(None));

/// Limit switch Y
static Y_LIMIT: Mutex<Cell<Option<Pin<Input<PullUp>, Dynamic>>>> = Mutex::new(Cell::new(None));

/// Limit switch Z
static Z_LIMIT: Mutex<Cell<Option<Pin<Input<PullUp>, Dynamic>>>> = Mutex::new(Cell::new(None));

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

    let y_stepper_pulse = pins.d24.into_output().downgrade();

    let y_stepper_direction = pins.d25.into_output().downgrade();

    let y_stepper_pulse_inverted = pins.d26.into_output().downgrade();

    let y_stepper_direction_inverted = pins.d27.into_output().downgrade();

    let z_stepper_pulse = pins.d28.into_output().downgrade();

    let z_stepper_direction = pins.d29.into_output().downgrade();

    let timer3 = Timer3Pwm::new(dp.TC3, Prescale64);

    #[allow(unused_mut)]
    let mut claw_pwm = pins.d5.into_output().into_pwm(&timer3);

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

        B_START
            .borrow(cs)
            .set(Some(pins.d15.into_pull_up_input().downgrade()));
        B_END
            .borrow(cs)
            .set(Some(pins.d14.into_pull_up_input().downgrade()));

        X_LIMIT
            .borrow(cs)
            .set(Some(pins.a8.into_pull_up_input().downgrade()));
        Y_LIMIT
            .borrow(cs)
            .set(Some(pins.a9.into_pull_up_input().downgrade()));
    });

    // enable interrupts for the device
    unsafe { interrupt::enable() };

    let exint = dp.EXINT;

    let mut game = Game::new(exint);
    game.run(
        x_stepper_pulse,
        x_stepper_direction,
        y_stepper_pulse,
        y_stepper_direction,
        y_stepper_pulse_inverted,
        y_stepper_direction_inverted,
        z_stepper_pulse,
        z_stepper_direction,
        claw_pwm,
    );
}
