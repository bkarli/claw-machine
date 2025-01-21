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
mod limit_switch;



#[allow(unused_imports)]
use panic_halt as _;

use crate::timer::{delay_s, delay_us, GenericTicker, PrecisionTicker};
use arduino_hal::hal::port::{Dynamic, PA0, PA1, PA2, PA3, PA4, PA5};
use arduino_hal::port::mode::{Input, Output, PullUp};
use arduino_hal::port::Pin;
use arduino_hal::simple_pwm::{IntoPwmPin};
use avr_device::interrupt;
use core::cell::{Cell, RefCell};
use core::pin::pin;
use embedded_hal::digital::OutputPin;
use futures::select_biased;
use crate::channel::{Channel, Receiver, Sender};
use crate::joystick::{JoystickDirection, JoystickSwitch};
use crate::stepper::{StepperDirection};
use crate::stepper::StepperDirection::{ClockWise, CounterClockWise, Idle};
use futures::FutureExt;

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

static X_LIMIT: Mutex<RefCell<Option<Pin<Input<PullUp>, Dynamic>>>> = Mutex::new(RefCell::new(None));

static Y_LIMIT: Mutex<RefCell<Option<Pin<Input<PullUp>, Dynamic>>>> = Mutex::new(RefCell::new(None));

static Z_LIMIT: Mutex<RefCell<Option<Pin<Input<PullUp>, Dynamic>>>> = Mutex::new(RefCell::new(None));




/// Create a console that can be used safely within an interrupt
static CONSOLE: Mutex<RefCell<Option<Console>>> = Mutex::new(RefCell::new(None));
const MAX_X_STEPS: i32 = 800;
const MAX_Y_STEPS: i32 = 800;
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

    let x_stepper_direction = pins.d27.into_output();
    let x_stepper_pulse = pins.d26.into_output();



    let y_stepper_direction = pins.d22.into_output();
    let y_stepper_pulse = pins.d23.into_output();

    let y_stepper_direction_inverted = pins.d24.into_output();
    let y_stepper_pulse_inverted = pins.d25.into_output();

    let z_stepper_direction = pins.d31.into_output();
    let z_stepper_pulse = pins.d30.into_output();


    // even tough interrupts are not enabled yet still have to create critical section for mutex
    // set all static variables
    interrupt::free(|cs| {
        // set console
        *CONSOLE.borrow(cs).borrow_mut() = Some(serial);

        // set input pins
        *J_RIGHT.borrow(cs).borrow_mut() = Some(pins.d50.into_pull_up_input().downgrade());
        *J_LEFT.borrow(cs).borrow_mut() = Some(pins.d52.into_pull_up_input().downgrade());
        *J_FORWARD.borrow(cs).borrow_mut() = Some(pins.d51.into_pull_up_input().downgrade());
        *J_BACKWARD.borrow(cs).borrow_mut() = Some(pins.d53.into_pull_up_input().downgrade());

        *X_LIMIT.borrow(cs).borrow_mut() = Some(pins.a8.into_pull_up_input().downgrade());

        *Y_LIMIT.borrow(cs).borrow_mut() = Some(pins.a9.into_pull_up_input().downgrade());

        *Z_LIMIT.borrow(cs).borrow_mut() = Some(pins.a10.into_pull_up_input().downgrade());

    });

    // enable interrupts for the device
    unsafe { interrupt::enable() };

    let exint = dp.EXINT;
    exint.pcicr.write(|w| unsafe { w.bits(0b011) });
    // Joystick pc interrupt pins
    exint.pcmsk0.write(|w| w.bits(0b00001111));
    let x_channel: Channel<StepperDirection> = Channel::new();
    let y_channel: Channel<StepperDirection> = Channel::new();
    let joystick_right_task = pin!(joystick_switch_task(
                        JoystickDirection::LEFT,
                        x_channel.get_sender()
                    ));
    let joystick_left_task = pin!(joystick_switch_task(
                        JoystickDirection::RIGHT,
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


    let y_stepper_task = pin!(y_gantry(y_channel.get_receiver(), y_stepper_pulse, y_stepper_pulse_inverted, y_stepper_direction, y_stepper_direction_inverted));
    let x_stepper_task = pin!(x_gantry(x_channel.get_receiver(),x_stepper_pulse, x_stepper_direction));
    executor::run_task(&mut [joystick_right_task,joystick_left_task,x_stepper_task, joystick_forward_task, joystick_backward_task, y_stepper_task])
}


pub async fn x_gantry (
    mut receiver: Receiver<'_,StepperDirection>,
    mut x_stepper_direction: Pin<Output, PA4>,
    mut x_stepper_pulse: Pin<Output, PA5>
) {
    let mut steps = 0;
    let mut stepper_direction = Idle;
    loop {
        match stepper_direction {
            Idle => {
                stepper_direction = receiver.receive().await;
            },
            ClockWise => {
                select_biased! {
                    new_direction = receiver.receive().fuse() => {stepper_direction = new_direction;},
                    _ = async {
                        if steps < MAX_X_STEPS {
                            x_stepper_direction.set_low();
                            x_stepper_pulse.set_high();
                            delay_us(1000).await;
                            x_stepper_pulse.set_low();
                            delay_us(1000).await;
                            x_stepper_direction.set_high();
                            steps += 1;
                        }else {
                            stepper_direction = Idle;
                        }

                    }.fuse() => {}
                }
            },
            CounterClockWise => {
                select_biased! {
                    new_direction = receiver.receive().fuse() => {stepper_direction = new_direction},
                    _ = async {
                        if steps >= 0 {
                            x_stepper_direction.set_high();
                            x_stepper_pulse.set_high();
                            delay_us(1000).await;
                            x_stepper_pulse.set_low();
                            delay_us(1000).await;
                            x_stepper_direction.set_low();
                            steps -= 1;

                        } else {
                            stepper_direction = Idle;
                        }

                    }.fuse() => {}
                }
            }
        }
    }
}
async fn joystick_switch_task(
    direction: JoystickDirection,
    motor_sender: Sender<'_, StepperDirection>,
) {
    let (index, stepper_direction): (usize, StepperDirection) = match direction {
        JoystickDirection::RIGHT => (3usize, ClockWise),
        JoystickDirection::LEFT => (2usize, CounterClockWise),
        JoystickDirection::FORWARD => (0usize, CounterClockWise),
        JoystickDirection::BACKWARD => (1usize, ClockWise),
    };

    let mut joystick_switch = JoystickSwitch::new(direction, index);

    loop {
        // wait for a low state
        joystick_switch.wait_for(false).await;
        motor_sender.send(stepper_direction);
        delay_us(300).await;


        // wait for a high state
        joystick_switch.wait_for(true).await;
        motor_sender.send(Idle);
    }
}

async fn y_gantry(
    mut receiver: Receiver<'_, StepperDirection>,
    mut y_stepper_pulse: Pin<Output, PA1>,
    mut y_stepper_pulse_inverted:  Pin<Output, PA3>,
    mut y_stepper_direction:  Pin<Output, PA0>,
    mut y_stepper_direction_inverted: Pin<Output, PA2>
) {
    let mut steps = 0;
    let mut stepper_direction = Idle;
    loop {
        match stepper_direction {
            Idle => {
                stepper_direction = receiver.receive().await;
            },
            ClockWise => {
                select_biased! {
                    new_direction = receiver.receive().fuse() => {stepper_direction = new_direction;},
                    _ = async {
                        if steps < MAX_Y_STEPS {
                            y_stepper_direction_inverted.set_high();
                            y_stepper_direction.set_high();
                            y_stepper_pulse.set_high();
                            y_stepper_pulse_inverted.set_high();
                            delay_us(1000).await;
                            y_stepper_pulse.set_low();
                            y_stepper_pulse_inverted.set_low();
                            delay_us(1000).await;
                            y_stepper_direction_inverted.set_low();
                            y_stepper_direction.set_low();
                            steps += 1;
                        }else {
                            stepper_direction = Idle;
                        }

                    }.fuse() => {}
                }
            },
            CounterClockWise => {
                select_biased! {
                    new_direction = receiver.receive().fuse() => {stepper_direction = new_direction},
                    _ = async {
                        if steps >= 0 {
                            y_stepper_direction_inverted.set_low();
                            y_stepper_direction.set_low();
                            y_stepper_pulse.set_high();
                            y_stepper_pulse_inverted.set_high();
                            delay_us(1000).await;
                            y_stepper_pulse.set_low();
                            y_stepper_pulse_inverted.set_low();
                            delay_us(1000).await;
                            y_stepper_direction_inverted.set_low();
                            y_stepper_direction.set_low();
                            steps -= 1;

                        } else {
                            stepper_direction = Idle;
                        }
                    }.fuse() => {}
                }
            }
        }
    }
}

async fn z_axis(

) {
    for _ in 0..1500 {

    }
}

