use avr_device::interrupt;
use core::cell::RefCell;
use core::fmt;
use core::future::poll_fn;
use core::task::Poll;
use ufmt::derive::uDebug;
use ufmt::{uWrite, Formatter};
use crate::executor::{wake_task, ExtWaker};
use crate::stepper::StepperDirection;
use crate::stepper::StepperDirection::{ClockWise, CounterClockWise, Idle};
use crate::{Mutex, J_BACKWARD, J_FORWARD, J_LEFT, J_RIGHT};

static JOYSTICK_SWITCH_TASKS: [Mutex<RefCell<usize>>; 4] = [
    Mutex::new(RefCell::new(0xFFFF)),
    Mutex::new(RefCell::new(0xFFFF)),
    Mutex::new(RefCell::new(0xFFFF)),
    Mutex::new(RefCell::new(0xFFFF)),
];

static JOYSTICK_SWITCH_STATES: Mutex<RefCell<[bool; 4]>> =
    Mutex::new(RefCell::new([false, false, false, false]));
pub struct JoystickSwitch {
    joystick_direction: JoystickDirection,
    switch_index: usize,
}

#[derive(Clone, Copy)]
pub enum JoystickDirection {
    RIGHT,
    LEFT,
    FORWARD,
    BACKWARD,
}


impl JoystickSwitch {
    pub fn new(joystick_direction: JoystickDirection, switch_index: usize) -> Self {
        Self {
            joystick_direction,
            switch_index,
        }
    }

    pub async fn wait_for(&mut self, desired_state: bool) {
        poll_fn(|cx| {
            if interrupt::free(|cs| {
                return match self.joystick_direction {
                    JoystickDirection::RIGHT => {
                        desired_state == J_RIGHT.borrow(cs).take().unwrap().is_high()
                    }
                    JoystickDirection::LEFT => {
                        desired_state == J_LEFT.borrow(cs).take().unwrap().is_high()
                    }
                    JoystickDirection::FORWARD => {
                        desired_state == J_FORWARD.borrow(cs).take().unwrap().is_high()
                    }
                    JoystickDirection::BACKWARD => {
                        desired_state == J_BACKWARD.borrow(cs).take().unwrap().is_high()
                    }
                };
            }) {
                Poll::Ready(())
            } else {
                interrupt::free(|cs| {
                    let _ = JOYSTICK_SWITCH_TASKS
                        .get(self.switch_index)
                        .unwrap()
                        .borrow(cs)
                        .replace(cx.waker().task_id());
                });
                Poll::Pending
            }
        })
        .await
    }
}

/**
Pin Change interrupt triggered if a Joy Stick switch has been triggered
*/
#[avr_device::interrupt(atmega2560)]
#[allow(non_snake_case)]
fn PCINT0() {
    // We don't actually need to create a critical section as AVR suppresses other interrupts during
    // an interrupt
    interrupt::free(|cs| {
        let right_pin = J_RIGHT.borrow(cs).take().unwrap().downgrade();
        let left_pin = J_LEFT.borrow(cs).take().unwrap().downgrade();
        let forward_pin = J_FORWARD.borrow(cs).take().unwrap().downgrade();
        let backward_pin = J_BACKWARD.borrow(cs).take().unwrap().downgrade();
        let joystick_states = JOYSTICK_SWITCH_STATES.borrow(cs).borrow_mut();
        let pins = [right_pin, left_pin, forward_pin, backward_pin];

        for (index, switch_state) in joystick_states.iter().enumerate() {
            if pins.get(index).unwrap().is_high() != *switch_state {
                let joystick_task = JOYSTICK_SWITCH_TASKS
                    .get(index)
                    .unwrap()
                    .borrow(cs)
                    .replace(0xFFFF);
                if joystick_task != 0xFFFF {
                    wake_task(joystick_task)
                }
            }
        }
    });
}

