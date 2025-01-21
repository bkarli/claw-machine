//!
//!
//!

use crate::channel::Receiver;
use crate::stepper::StepperDirection::{ClockWise, CounterClockWise, Idle};
use crate::timer::delay_us;
use arduino_hal::hal::port::{Dynamic, PA0, PA1, PA2, PA3, PA4, PA5};
use arduino_hal::port::mode::Output;
use arduino_hal::port::Pin;
use core::future::join;
use futures::FutureExt;

use embedded_hal::digital::OutputPin;
use futures::select_biased;

const MAX_X_STEPS: i32 = 1000;
const MAX_Y_STEPS: i32 = 1000;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StepperDirection {
    Idle,
    ClockWise,
    CounterClockWise,
}


pub async fn x_gantry (
    mut receiver: Receiver<'_,StepperDirection>,
    x_stepper_direction: &mut Pin<Output, PA1>,
    x_stepper_pulse: &mut Pin<Output, PA0>
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
                    stepper_direction = receiver.receive().fuse() => {},
                    _ = async {
                        if steps > MAX_X_STEPS {
                            return;
                        }
                        x_stepper_pulse.set_high();
                        delay_us(1000).await;
                        x_stepper_pulse.set_low();
                        delay_us(1000).await;
                        steps += 1;
                    }.fuse() => {}
                }
            },
            CounterClockWise => {
                select_biased! {
                    stepper_direction = receiver.receive().fuse() => {},
                    _ = async {
                        if steps <= 0 {
                            return;
                        }
                        x_stepper_direction.set_high();
                        x_stepper_pulse.set_high();
                        delay_us(1000).await;
                        x_stepper_pulse.set_low();
                        delay_us(1000).await;
                        x_stepper_direction.set_low();
                        steps -= 1;
                    }.fuse() => {}
                }
            }
        }
    }
}

pub async fn y_gantry (
    mut receiver: Receiver<'_,StepperDirection>,
    y_stepper_direction: &mut Pin<Output, PA3>,
    y_stepper_pulse: &mut Pin<Output, PA2>,
    y_stepper_direction_inverted: &mut Pin<Output, PA5>,
    y_stepper_pulse_inverted: &mut Pin<Output, PA4>
) {
    let mut steps = 0;
    loop {
        let mut stepper_direction = Idle;
        loop {
            match stepper_direction {
                Idle => {
                    stepper_direction = receiver.receive().await;
                },
                ClockWise => {
                    select_biased! {
                    stepper_direction = receiver.receive().fuse() => {},
                    _ = async {
                        if steps > MAX_X_STEPS {
                            return;
                        }
                        y_stepper_direction_inverted.set_high();
                        y_stepper_pulse.set_high();
                        y_stepper_pulse_inverted.set_high();
                        delay_us(1000).await;
                        y_stepper_pulse.set_low();
                        y_stepper_pulse_inverted.set_low();
                        delay_us(1000).await;
                        y_stepper_direction_inverted.set_low();
                        steps += 1;
                    }.fuse() => {}
                }
                },
                CounterClockWise => {
                    select_biased! {
                    stepper_direction = receiver.receive().fuse() => {},
                    _ = async {
                        if steps <= 0 {
                            return;
                        }
                        y_stepper_direction.set_high();
                        y_stepper_pulse.set_high();
                        y_stepper_pulse_inverted.set_high();
                        delay_us(1000).await;
                        y_stepper_pulse.set_low();
                        y_stepper_pulse_inverted.set_low();
                        delay_us(1000).await;
                        y_stepper_direction.set_low();
                        steps -= 1;
                    }.fuse() => {}
                }
                }
            }
        }
    }
}