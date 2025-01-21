//!
//!
//!

use crate::channel::Receiver;
use crate::stepper::StepperDirection::{ClockWise, CounterClockWise, Idle};
use crate::timer::delay_us;
use arduino_hal::hal::port::Dynamic;
use arduino_hal::port::mode::Output;
use arduino_hal::port::Pin;
use core::future::join;
use embedded_hal::digital::OutputPin;
use futures::select_biased;

const MAX_STEPS: u32 = 6000;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StepperDirection {
    Idle,
    ClockWise,
    CounterClockWise,
}
pub struct Stepper {
    direction_pin: Pin<Output, Dynamic>,
    pulse_pin: Pin<Output, Dynamic>,
    inverted: bool,
    max: u32,
}

impl Stepper {
    pub fn new(
        direction_pin: Pin<Output, Dynamic>,
        pulse_pin: Pin<Output, Dynamic>,
        inverted: bool,
    ) -> Self {
        let max: u32 = if inverted { MAX_STEPS } else { 0u32 };
        Self {
            direction_pin,
            pulse_pin,
            inverted,
            max,
        }
    }

    /**
    Steps the motor in one direction with a pulse width or variable timeout
    */
    pub async fn move_direction(&mut self, direction: StepperDirection, pulse_width: u32) {
        match direction {
            ClockWise => {
                if self.max > MAX_STEPS {
                    return;
                }
                self.pulse_pin.set_high();
                delay_us(pulse_width).await;
                self.pulse_pin.set_low();
                self.max += 1
            }
            CounterClockWise => {
                if self.max <= 0 {
                    return;
                }
                self.direction_pin.set_high();
                self.pulse_pin.set_high();
                delay_us(pulse_width).await;
                self.pulse_pin.set_low();
                self.direction_pin.set_low();
                self.max -= 1
            }
            _ => {}
        }
    }
}

pub async fn stepper_task_x(
    stepper_pin: Pin<Output>,
    direction_pin: Pin<Output>,
    mut receiver: Receiver<'_, StepperDirection>,
) {
    let mut motor = Stepper::new(stepper_pin, direction_pin, false);
    loop {
        let direction = receiver.receive().await;
    }
}

pub async fn stepper_task_y(
    stepper_pin_1: Pin<Output, Dynamic>,
    direction_pin_1: Pin<Output, Dynamic>,

    stepper_pin_2: Pin<Output, Dynamic>,
    direction_pin_2: Pin<Output, Dynamic>,
    mut axis_receiver: Receiver<'_, StepperDirection>,
) {
    let mut motor_1 = Stepper::new(stepper_pin_1, direction_pin_1, false);
    let mut motor_2 = Stepper::new(stepper_pin_2, direction_pin_2, true);

    // should be idle at start
    let mut direction: StepperDirection = Idle;
    let mut inverted_direction = Idle;

    loop {
        match direction {
            Idle => {
                // current state is idle wait for a new direction
                direction = axis_receiver.receive().await;
                // once new direction has been received invert it
                inverted_direction = match direction {
                    ClockWise => CounterClockWise,
                    CounterClockWise => ClockWise,
                    Idle => Idle,
                };
            }
            _ => {
                // await both futures at the same time
                join!(
                    motor_1.move_direction(direction, 1000),
                    motor_2.move_direction(inverted_direction, 1000)
                )
                .await;
            }
        }
    }
}
