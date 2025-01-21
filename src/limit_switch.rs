use crate::executor::{wake_task, ExtWaker};
use crate::{Mutex, X_LIMIT, Y_LIMIT, Z_LIMIT};
use arduino_hal::hal::port::Dynamic;
use arduino_hal::port::mode::{Input, PullUp};
use arduino_hal::port::Pin;
use avr_device::interrupt;
use core::cell::{Cell, RefCell};
use core::future::poll_fn;
use core::task::Poll;

/// a static array that holds the waker ids
/// Gets initialized with an invalid waker ID
///
/// Indices
/// 0: X-Limit
/// 1: Y-Limit
/// 2: Z-Limit
static LIMIT_SWITCH_TASKS: [Mutex<RefCell<usize>>; 3] = [
    Mutex::new(Cell::new(0xFFFF)),
    Mutex::new(Cell::new(0xFFFF)),
    Mutex::new(Cell::new(0xFFFF)),
];

/// a static array that holds the previous state of the limit switches
/// If the previous state differs from the current state the specific switch triggered the interrupt
/// true = high
/// false = low
/// Indices
/// 0: X-Limit
/// 1: Y-Limit
/// 2: Z-Limit
static LIMIT_SWITCH_STATES: Mutex<RefCell<[bool; 3]>> =
    Mutex::new(RefCell::new([true, true, true]));

/**
struct for the limit switches
*/
struct LimitSwitch {
    switch_index: usize,

}

impl LimitSwitch {
    pub fn new(switch_index: usize) -> Self {
        Self { switch_index }
    }

    pub async fn wait_for(&mut self, desired_state: bool) {
        poll_fn(|cx| {
            if interrupt::free(|cs| {
                // 0 => X
                // 1 => Y
                // 2 => Z
                return match self.switch_index {
                    0 => desired_state == X_LIMIT.borrow(cs).take().unwrap().is_high(),
                    1 => desired_state == Y_LIMIT.borrow(cs).take().unwrap().is_high(),
                    2 => desired_state == Z_LIMIT.borrow(cs).take().unwrap().is_high(),
                    _ => return false,
                };
            }) {
                Poll::Ready(())
            } else {
                interrupt::free(|cs| {
                    LIMIT_SWITCH_TASKS[self.switch_index]
                        .borrow(cs)
                        .replace(cx.waker().task())
                });
                Poll::Pending
            }
        })
        .await
    }
}

/**
Pin Change interrupt triggered if a limit switch has been triggered
*/
#[avr_device::interrupt(atmega2560)]
#[allow(non_snake_case)]
fn PCINT2() {
    // We don't actually need to create a critical section as AVR suppresses other interrupts during
    // an Interrupt
    interrupt::free(|cs| {
        let limit_x_pin = X_LIMIT.borrow(cs).take().unwrap();
        let limit_y_pin = Y_LIMIT.borrow(cs).take().unwrap();
        let limit_z_pin = Z_LIMIT.borrow(cs).take().unwrap();

        let switches = [
            limit_x_pin.downgrade(),
            limit_y_pin.downgrade(),
            limit_z_pin.downgrade(),
        ];
        let switch_states = LIMIT_SWITCH_STATES.borrow(cs).borrow_mut();

        for (index, switch_state) in switch_states.iter().enumerate() {
            if switches.get(index).unwrap().is_high() != *switch_state {
                let limit_switch_task = LIMIT_SWITCH_TASKS
                    .get(index)
                    .unwrap()
                    .borrow(cs)
                    .replace(0xFFFF);
                if limit_switch_task != 0xFFFF {
                    wake_task(limit_switch_task)
                }
            }
        }
    });
}
