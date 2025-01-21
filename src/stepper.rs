//!
//!
//!

use crate::channel::Receiver;
use crate::stepper::StepperDirection::{ClockWise, CounterClockWise, Idle};
use crate::timer::delay_us;
use arduino_hal::hal::port::{Dynamic, PA4, PA5};
use arduino_hal::port::mode::Output;
use arduino_hal::port::Pin;
use core::future::join;
use embedded_hal::digital::OutputPin;
use futures::select_biased;

const MAX_STEPS: u32 = 400;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StepperDirection {
    Idle,
    ClockWise,
    CounterClockWise,
}
pub struct Stepper {
    direction_pin: Pin<Output, PA4>,
    pulse_pin: Pin<Output, PA5>,
    inverted: bool,
    max: u32,
}

impl Stepper {
    pub fn new(
        direction_pin: Pin<Output, PA4>,
        pulse_pin: Pin<Output, PA5>,
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
                delay_us(pulse_width).await;
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
                delay_us(pulse_width).await;
                self.direction_pin.set_low();
                self.max -= 1
            }
            _ => {}
        }
    }
}
