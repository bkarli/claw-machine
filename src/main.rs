//! Main File for the claw machine
//!
//!

#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
#![feature(future_join)]

mod executor;
mod joystick;
mod stepper;

#[allow(unused_imports)]
use panic_halt as _;

use arduino_hal::hal::port::Dynamic;
use arduino_hal::port::mode::{Input, PullUp};
use arduino_hal::port::Pin;

use arduino_hal::simple_pwm::{IntoPwmPin};
use avr_device::interrupt;
use core::cell::{Cell, RefCell};
use core::pin::pin;
use avr_device::interrupt::CriticalSection;
use crate::joystick::{JoystickDirection, JoystickSwitch};
use crate::stepper::StepperDirection;
use crate::stepper::StepperDirection::{ClockWise, CounterClockWise};

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

    // create all joystick tasks
    let joystick_right_task = pin!(test_joy_stick(
                        JoystickDirection::RIGHT,
                    ));
    let joystick_left_task = pin!(test_joy_stick(
                        JoystickDirection::LEFT,
                    ));
    let joystick_forward_task = pin!(test_joy_stick(
                        JoystickDirection::FORWARD,
                    ));
    let joystick_backward_task = pin!(test_joy_stick(
                        JoystickDirection::BACKWARD,
                    ));


    // enable interrupts for the device
    unsafe { interrupt::enable() }
    let exint = dp.EXINT;
    exint.pcicr.write(|w| unsafe { w.bits(0b011) });
    // Joystick pc interrupt pins
    exint.pcmsk0.write(|w| w.bits(0b00001111));

    executor::run_task(&mut [joystick_right_task,joystick_left_task,joystick_forward_task,joystick_backward_task])


}

pub async fn test_joy_stick(
    direction: JoystickDirection,
) {
    let (index, stepper_direction): (usize, StepperDirection) = match direction {
        JoystickDirection::RIGHT => (0usize, CounterClockWise),
        JoystickDirection::LEFT => (1usize, ClockWise),
        JoystickDirection::FORWARD => (2usize, CounterClockWise),
        JoystickDirection::BACKWARD => (3usize, ClockWise),
    };

    let mut joystick_switch = JoystickSwitch::new(direction, index);

    loop {
        // wait for a low state
        joystick_switch.wait_for(false).await;
        interrupt::free(|cs| {
            if let Some(console) = CONSOLE.borrow(cs).borrow_mut().as_mut() {
                match direction {
                    JoystickDirection::RIGHT => {
                        let _ = ufmt::uwriteln!(console, "Right pressed");
                    }
                    JoystickDirection::LEFT => {
                        let _ = ufmt::uwriteln!(console, "Left pressed");
                    }
                    JoystickDirection::FORWARD => {
                        let _ = ufmt::uwriteln!(console, "Forward pressed");
                    }
                    JoystickDirection::BACKWARD => {
                        let _ = ufmt::uwriteln!(console, "Backward pressed");
                    }
                }
            }
        });
        // todo debounce

        // wait for a high state
        joystick_switch.wait_for(true).await;
        interrupt::free(|cs| {
            if let Some(console) = CONSOLE.borrow(cs).borrow_mut().as_mut() {
                match direction {
                    JoystickDirection::RIGHT => {
                        let _ = ufmt::uwriteln!(console, "Right released");
                    }
                    JoystickDirection::LEFT => {
                        let _ = ufmt::uwriteln!(console, "Left released");
                    }
                    JoystickDirection::FORWARD => {
                        let _ = ufmt::uwriteln!(console, "Forward released");
                    }
                    JoystickDirection::BACKWARD => {
                        let _ = ufmt::uwriteln!(console, "Backward released");
                    }
                }
            }
        });

    }
}