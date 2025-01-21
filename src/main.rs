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

use crate::game::{Game, GameState};
use crate::timer::{GenericTicker, PrecisionTicker};
use arduino_hal::hal::port::Dynamic;
use arduino_hal::port::mode::{Input, PullUp};
use arduino_hal::port::Pin;
use arduino_hal::simple_pwm::Prescaler::Prescale64;
use arduino_hal::simple_pwm::{IntoPwmPin, Timer3Pwm};
use avr_device::interrupt;
use core::cell::{Cell, RefCell};
use core::pin::pin;
use crate::channel::Channel;
use crate::joystick::{joystick_switch_task, JoystickDirection};
use crate::stepper::{x_gantry, y_gantry, StepperDirection};

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
static J_RIGHT: Mutex<RefCell<Option<Pin<Input<PullUp>, Dynamic>>>> = Mutex::new(RefCell::new(None));

/// Joystick Left input Pin
static J_LEFT: Mutex<RefCell<Option<Pin<Input<PullUp>, Dynamic>>>> = Mutex::new(RefCell::new(None));

/// Joystick Forward input Pin
static J_FORWARD: Mutex<RefCell<Option<Pin<Input<PullUp>, Dynamic>>>> = Mutex::new(RefCell::new(None));

/// Joystick Backward input Pin
static J_BACKWARD: Mutex<RefCell<Option<Pin<Input<PullUp>, Dynamic>>>> = Mutex::new(RefCell::new(None));

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


    // create a serial connection with the console output
    let serial = arduino_hal::default_serial!(dp, pins, 57600);

    let mut x_stepper_pulse = pins.d22.into_output();

    let mut x_stepper_direction = pins.d23.into_output();

    let mut y_stepper_pulse = pins.d24.into_output();

    let mut y_stepper_direction = pins.d25.into_output();

    let mut y_stepper_pulse_inverted = pins.d26.into_output();

    let mut y_stepper_direction_inverted = pins.d27.into_output();

    let mut z_stepper_pulse = pins.d28.into_output();

    let mut  z_stepper_direction = pins.d29.into_output();

    let mut start_led = pins.d30.into_output();

    let mut end_led = pins.d31.into_output();

    let timer3 = Timer3Pwm::new(dp.TC3, Prescale64);

    #[allow(unused_mut)]
    let mut claw_pwm = pins.d5.into_output().into_pwm(&timer3);

    // even tough interrupts are not enabled yet still have to create critical section for mutex
    // set all static variables
    interrupt::free(|cs| {
        // set console
        *CONSOLE.borrow(cs).borrow_mut() = Some(serial);

        // set input pins
        *J_RIGHT.borrow(cs).borrow_mut() = Some(pins.d50.into_pull_up_input().downgrade());
        *J_LEFT.borrow(cs).borrow_mut() = Some(pins.d51.into_pull_up_input().downgrade());
        *J_FORWARD.borrow(cs).borrow_mut() = Some(pins.d52.into_pull_up_input().downgrade());
        *J_BACKWARD.borrow(cs).borrow_mut() = Some(pins.d53.into_pull_up_input().downgrade());

    });
    // initialize static Tickers
    PrecisionTicker::init(dp.TC0);
    GenericTicker::init(dp.TC1);
    // enable interrupts for the device
    unsafe { interrupt::enable() };

    let exint = dp.EXINT;

    let mut game_state = GameState::IDLE;


    // game loop
    loop {
        match game_state {
            GameState::IDLE => {
                // enable limit switch interrupts
                exint.pcicr.write(|w| unsafe { w.bits(0b100) });
                exint.pcmsk2.write(|w| w.bits(0b00000111));

                let reset_task = pin!(reset_game());
                let blink_led_task = pin!(blink_led());
                executor::run_task(&mut [reset_task, blink_led_task]);

                // enable UI button interrupts and disable limit switch interrupts
                exint.pcicr.write(|w| unsafe { w.bits(0b010) });
                exint.pcmsk1.write(|w| w.bits(0b00000010));

                // task that waits for user to press green button
                let wait_for_start_task = pin!(wait_for_start());
                executor::run_task(&mut [wait_for_start_task]);


                game_state = GameState::RUNNING;
            }
            GameState::RUNNING => {
                // enable all interrupts except limit switches
                exint.pcicr.write(|w| unsafe { w.bits(0b011) });
                // Joystick pc interrupt pins
                exint.pcmsk0.write(|w| w.bits(0b00001111));
                // end button interrupt pin
                exint.pcmsk1.write(|w| w.bits(0b00000100));


                // channels for both x-and y-axis
                let x_channel: Channel<StepperDirection> = Channel::new();
                let y_channel: Channel<StepperDirection> = Channel::new();

                let x_gantry_task = pin!(x_gantry(
                    x_channel.get_receiver(),
                    &mut x_stepper_direction,
                    &mut x_stepper_pulse
                ));

                let y_gantry_task = pin!(y_gantry(
                    y_channel.get_receiver(),
                    &mut y_stepper_direction,
                    &mut y_stepper_pulse,
                    &mut y_stepper_direction_inverted,
                    &mut y_stepper_pulse_inverted,
                ));

                let joystick_right_task = pin!(joystick_switch_task(
                        JoystickDirection::RIGHT,
                        x_channel.get_sender()
                    ));
                let joystick_left_task = pin!(joystick_switch_task(
                        JoystickDirection::LEFT,
                        x_channel.get_sender()
                    ));
                let joystick_forward_task = pin!(joystick_switch_task(
                        JoystickDirection::FORWARD,
                        y_channel.get_sender()
                    ));
                let joystick_backward_task = pin!(joystick_switch_task(
                        JoystickDirection::BACKWARD,
                        y_channel.get_sender()
                    ));

                executor::run_task(&mut [joystick_right_task,joystick_left_task,joystick_forward_task,joystick_backward_task, x_gantry_task, y_gantry_task ])

            },
            GameState::FINISHED => {
            }
        }
    }
}

async fn reset_game(

) {

}

async fn wait_for_start(

) {

}
async fn wait_for_end(

) {

}

async fn blink_led(

) {

}