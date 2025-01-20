use avr_device::interrupt;
use core::cell::{Cell, RefCell};
use core::fmt;
use core::future::poll_fn;
use core::task::Poll;
use avr_device::interrupt::CriticalSection;
use ufmt::derive::uDebug;
use ufmt::{uWrite, Formatter};
use crate::executor::{wake_task, ExtWaker};
use crate::stepper::StepperDirection;
use crate::stepper::StepperDirection::{ClockWise, CounterClockWise, Idle};
use crate::{Mutex, CONSOLE, J_BACKWARD, J_FORWARD, J_LEFT, J_RIGHT};

static JOYSTICK_SWITCH_TASKS: [Mutex<Cell<usize>>; 4] = [
    Mutex::new(Cell::new(0xFFFF)),
    Mutex::new(Cell::new(0xFFFF)),
    Mutex::new(Cell::new(0xFFFF)),
    Mutex::new(Cell::new(0xFFFF)),
];

/// all states should be high at the start
static JOYSTICK_SWITCH_STATES: Mutex<RefCell<[bool; 4]>> =
    Mutex::new(RefCell::new([true, true, true, true]));
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
                        desired_state == J_RIGHT.borrow(cs).borrow_mut().as_ref().unwrap().is_high()
                    }
                    JoystickDirection::LEFT => {
                        desired_state == J_LEFT.borrow(cs).borrow_mut().as_ref().unwrap().is_high()
                    }
                    JoystickDirection::FORWARD => {
                        desired_state == J_FORWARD.borrow(cs).borrow_mut().as_ref().unwrap().is_high()
                    }
                    JoystickDirection::BACKWARD => {
                        desired_state == J_BACKWARD.borrow(cs).borrow_mut().as_ref().unwrap().is_high()
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

                    // replace the desired state of the array
                    let mut states = JOYSTICK_SWITCH_STATES
                        .borrow(cs)
                        .borrow_mut();
                    states[self.switch_index] = desired_state;

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
        let states = JOYSTICK_SWITCH_STATES.borrow(cs);
        if let Some(right_pin) = J_RIGHT.borrow(cs).borrow_mut().as_ref() {
            let state = states.borrow().as_ref()[0];
            if right_pin.is_high() == states.borrow().as_ref()[0] {
                let task_id = JOYSTICK_SWITCH_TASKS[0].borrow(cs).replace(0xFFFF);
                if task_id != 0xFFFF {
                    wake_task(task_id)
                }
            }
        };
        if let Some(left_pin) = J_LEFT.borrow(cs).borrow_mut().as_ref(){
            let state = states.borrow().as_ref()[1];
            if left_pin.is_high() == state {
                let task_id = JOYSTICK_SWITCH_TASKS[1].borrow(cs).replace(0xFFFF);
                if task_id != 0xFFFF {
                    states.borrow_mut().as_mut()[3] != state;
                    wake_task(task_id)
                }
            }
        };
        if let Some(forward_pin) = J_FORWARD.borrow(cs).borrow_mut().as_ref(){
            let state = states.borrow().as_ref()[2];
            if forward_pin.is_high()== state {
                let task_id = JOYSTICK_SWITCH_TASKS[2].borrow(cs).replace(0xFFFF);
                if task_id != 0xFFFF {
                    wake_task(task_id)
                }
            }
        };
        if let Some(backward_pin) = J_BACKWARD.borrow(cs).borrow_mut().as_ref(){
            let state = states.borrow().as_ref()[3];
            if backward_pin.is_high() == state {
                let task_id = JOYSTICK_SWITCH_TASKS[3].borrow(cs).replace(0xFFFF);
                if task_id != 0xFFFF {
                    states.borrow_mut().as_mut()[3] != state;
                    wake_task(task_id)
                }
            }
        };
    });
}

