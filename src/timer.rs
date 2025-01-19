//! This module abstracts two AVR timers
//!
//! In our project we need three timers
//!
//! t0: is our precision timer it handles pulse generation for our servo motors
//! t1: is our generic timer which handles the automatic finish after 30 seconds
//! t2: will be used for PWM control of our servo motor, but we will use SimplePWM from the HAL
//!
//! Both timers are built in the way that we could expand the project, ex. when more components would
//! need a timer like LEDs or a count-down clock.
//!
//! How to use:
//! Create A Timer instance and add to its respective queue

use avr_device::atmega2560::{TC0, TC1};
use avr_device::interrupt;
use core::cell::{Cell, RefCell, RefMut};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use heapless::binary_heap::{BinaryHeap, Min};
use crate::CONSOLE;
use crate::executor::{wake_task, ExtWaker};

// Type alias for avr_device::interrupt::Mutex to Mutex
type Mutex<T> = interrupt::Mutex<T>;

/// declare static generic Ticker
static G_TICKER: GenericTicker = GenericTicker {
    tc1: Mutex::new(RefCell::new(None)),
    max: 62500,
};

/// binary heap that acts as the priority queue for our Generic timers
static G_QUEUE: Mutex<RefCell<BinaryHeap<(u64, usize), Min, 4>>> =
    Mutex::new(RefCell::new(BinaryHeap::new()));

/// Keep track of current generic tick count
static G_TICK_COUNTER: Mutex<Cell<u64>> = Mutex::new(Cell::new(0));

/// A variable tick incrementer
static G_TICK_INCREMENT: Mutex<Cell<u64>> = Mutex::new(Cell::new(65535));

/**
Constant conversion that convert seconds to generic ticks
Max register: 65535
Our max 62'500
When our max match occurs every four seconds
*/
const fn s_to_generic_ticks(s: u8) -> u64 {
    62500 * s as u64
}

pub enum TimerState {
    Init,
    Waiting,
}

/**
Ticker with seconds precision and used internally to Generate Timer Events with seconds precision
*/
pub struct GenericTicker {
    pub(crate) tc1: Mutex<RefCell<Option<TC1>>>,
    pub max: u16,
}

impl GenericTicker {
    pub fn init(tc1: TC1) {
        // write counter max to register
        tc1.ocr1a.write(|w| w.bits(62500));
        // set flag to only count to max and set CTC mode
        tc1.tccr1b.write(|w| {
            w.wgm1().bits(4);
            w.cs1().prescale_256()
        });
        // enable CTC mode interrupt
        tc1.timsk1.write(|w| w.ocie1a().set_bit());

        // replace the tc1
        interrupt::free(|cs| {
            G_TICKER.tc1.borrow(cs).replace(Some(tc1));
            G_TICK_COUNTER.borrow(cs).set(0);
        })
    }

    /**
    Gets the current generic tick count
    */
    pub fn now() -> u64 {
        interrupt::free(|cs| G_TICK_COUNTER.borrow(cs).get())
    }

    pub fn seconds() -> u64 {
        let ticks = Self::now();
        Self::millis_from_ticks(ticks)
    }

    fn millis_from_ticks(ticks: u64) -> u64 {
        ticks / 62500
    }

}

pub(crate) struct GenericTimer {
    end_ticks: u64,
    state: TimerState,
}

impl GenericTimer {
    pub fn new(seconds: u8) -> Self {
        Self {
            end_ticks: s_to_generic_ticks(seconds),
            state: TimerState::Init,
        }
    }
    pub fn register(&self, task: usize) {
        // create critical section as no interrupts should happen during registering of a timer
        // also we need some shared variables
        interrupt::free(|cs| {
            let mut queue = G_QUEUE.borrow(cs).borrow_mut();
            let is_first = if let Some((next_timer, _)) = queue.peek() {
                self.end_ticks < *next_timer
            } else {
                true
            };
            if queue.push((self.end_ticks, task)).is_err() {
                panic!("Queue full")
            }
            // if it is the first element queue for wakeup
            if is_first {
                let ticks = G_TICK_COUNTER.borrow(cs).get();
                let increment_c = G_TICK_INCREMENT.borrow(cs);
                schedule_generic_wakeup(
                    queue,
                    G_TICKER.tc1.borrow(cs).borrow_mut(),
                    ticks,
                    increment_c,
                )
            }
        })
    }
}

impl Future for GenericTimer {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.state {
            TimerState::Init => {
                self.register(cx.waker().task_id());
                self.state = TimerState::Waiting;
                Poll::Pending
            }
            TimerState::Waiting => {
                if GenericTicker::now() >= self.end_ticks {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            }
        }
    }
}

/**
public function that creates a GenericTimer that delays something for n seconds
*/
pub async fn delay_s(seconds: u8) {
    GenericTimer::new(seconds).await
}

/**
Add a new timer to the generic queue
*/
fn schedule_generic_wakeup(
    mut queue: RefMut<BinaryHeap<(u64, usize), Min, 4>>,
    mut tc1: RefMut<Option<TC1>>,
    counter: u64,
    increment_c: &Cell<u64>,
) {
    // take first of queue
    // if first end tick - current tick <= 65535 add set ticker delta and modify increment
    while let Some((end_ticks, task)) = queue.peek() {
        let remainder: u64 = *end_ticks - counter;
        if remainder <= 65535 {
            // if ticks are near 0 another interrupt is not necessary => Wake up task immediately
            if remainder <= 10 {
                wake_task(*task);

                // remove timer from queue
                queue.pop();
                continue;
            } else {
                // create a timed interrupt for the remaining time
                tc1.as_mut()
                    .unwrap()
                    .ocr1a
                    .write(|w| w.bits(remainder as u16));

                // update the increment amount
                increment_c.set(remainder);
                // new timer has been scheduled break the while loop
                break;
            }
        }
        break;
    }
}

/**
Interrupt triggered at least every seconds
*/
#[avr_device::interrupt(atmega2560)]
#[allow(non_snake_case)]
fn TIMER1_COMPA() {
    interrupt::free(|cs| {
        let counter_c = G_TICK_COUNTER.borrow(cs);
        let increment_c = G_TICK_INCREMENT.borrow(cs);
        let counter = counter_c.get() + G_TICK_INCREMENT.borrow(cs).get();
        if let Some(console) = CONSOLE.borrow(cs).borrow_mut().as_mut() {
            let _ = ufmt::uwriteln!(console,"interrupt triggered");
        }

        counter_c.set(counter);
        schedule_generic_wakeup(
            G_QUEUE.borrow(cs).borrow_mut(),
            G_TICKER.tc1.borrow(cs).borrow_mut(),
            counter,
            increment_c,
        )
    })
}

/**
Constant conversion that convert microseconds to precision ticks

250 ticks equals one millisecond

Min is 1 Tick which is 2.5 microseconds
*/
const fn us_to_p_ticks(us: u16) -> u16 {
    250 * us / 1000
}

/// declare static precision ticker
static P_TICKER: PrecisionTicker = PrecisionTicker {
    tc0: Mutex::new(RefCell::new(None)),
    max: 250,
};

/// binary heap that acts as the priority queue for our Precision timers
static P_QUEUE: Mutex<RefCell<BinaryHeap<(u64, usize), Min, 8>>> =
    Mutex::new(RefCell::new(BinaryHeap::new()));

/// Keep track of current precision tick count
static P_TICK_COUNTER: Mutex<Cell<u64>> = Mutex::new(Cell::new(0));

/// A variable tick incrementer
static P_TICK_INCREMENT: Mutex<Cell<u64>> = Mutex::new(Cell::new(65535));

/**
Ticker to generate Ticker with microsecond precision used internally to generate Timer Events with
microsecond precision.

we will be using delays between 300..3000 microseconds to accelerate and decelerate our motors, so
ideally we use some sort of combination of prescaler and register value that overflows or triggers
every 100 microseconds

with a prescaler of 64 and a max value of 250 we know that the timer will at least overflow every ms

*/
pub struct PrecisionTicker {
    pub(crate) tc0: Mutex<RefCell<Option<TC0>>>,
    pub max: u8,
}

impl PrecisionTicker {
    fn now() -> u64 {
        interrupt::free(|cs| P_TICK_COUNTER.borrow(cs).get())
    }

    pub fn millis() -> u64 {
        let ticks = Self::now();
        Self::from_ticks_to_millis(ticks)
    }

    fn from_ticks_to_millis(ticks: u64) -> u64 {
        ticks / 250
    }
}

impl PrecisionTicker {
    pub fn init(tc0: TC0) {
        // enable CTC (clear timer on compare match)
        tc0.tccr0a.write(|w| w.wgm0().ctc());
        // choose the prescaler of the counter register
        tc0.tccr0b.write(|w| w.cs0().prescale_64());

        // replace tc0
        interrupt::free(|cs| {
            P_TICKER.tc0.borrow(cs).replace(Some(tc0));
        })
    }
}

pub struct PrecisionTimer {
    end_ticks: u64,
    state: TimerState,
}

impl PrecisionTimer {
    pub fn new(microseconds: u16) -> Self {
        Self {
            end_ticks: us_to_p_ticks(microseconds) as u64,
            state: TimerState::Init,
        }
    }

    fn register(&self, task: usize) {
        interrupt::free(|cs| {
            let mut queue = P_QUEUE.borrow(cs).borrow_mut();
            let is_first = if let Some((next_timer, _)) = queue.peek() {
                self.end_ticks < *next_timer
            } else {
                true
            };
            if queue.push((self.end_ticks, task)).is_err() {
                panic!("Queue full")
            }
            // if it is the first element queue for wakeup
            if is_first {
                let ticks = P_TICK_COUNTER.borrow(cs).get();
                let increment_c = P_TICK_INCREMENT.borrow(cs);
                schedule_precision_wakeup(
                    queue,
                    P_TICKER.tc0.borrow(cs).borrow_mut(),
                    ticks,
                    increment_c,
                )
            }
        })
    }
}

impl Future for PrecisionTimer {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.state {
            TimerState::Init => {
                self.register(cx.waker().task_id());
                self.state = TimerState::Waiting;
                Poll::Pending
            }
            TimerState::Waiting => {
                if PrecisionTicker::now() >= self.end_ticks {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            }
        }
    }
}
fn schedule_precision_wakeup(
    mut queue: RefMut<BinaryHeap<(u64, usize), Min, 8>>,
    mut tc0: RefMut<Option<TC0>>,
    counter: u64,
    increment_c: &Cell<u64>)
{
    // take first of queue
    // if first end tick - current tick <= 65535 add set ticker delta and modify increment
    while let Some((end_ticks, task)) = queue.peek() {
        let remainder: u64 = *end_ticks - counter;

        if remainder <= 250 {
            // if ticks are near 0 another interrupt is not necessary => Wake up task immediately
            if remainder <= 5 {

                wake_task(*task);

                // remove timer from queue
                queue.pop();
            } else {
                // create a timed interrupt for the remaining time
                tc0.as_mut()
                    .unwrap()
                    .ocr0a
                    .write(|w| w.bits(remainder as u8));

                // update the increment amount
                increment_c.set(remainder);
                // new timer has been scheduled break the while loop
                break;
            }
        }
        break;
    }

}

/**
Public function that delays something for n us
*/
pub async fn delay_us(us: u16) {
    PrecisionTimer::new(us).await
}

/**
Interrupt triggered at least every millisecond
*/
#[avr_device::interrupt(atmega2560)]
#[allow(non_snake_case)]
fn TIMER0_COMPA() {
    interrupt::free(|cs| {
        let counter_c = P_TICK_COUNTER.borrow(cs);
        let increment_c = P_TICK_INCREMENT.borrow(cs);
        let counter = counter_c.get() + P_TICK_INCREMENT.borrow(cs).get();
        counter_c.set(counter);
        schedule_precision_wakeup(
            P_QUEUE.borrow(cs).borrow_mut(),
            P_TICKER.tc0.borrow(cs).borrow_mut(),
            counter,
            increment_c,
        )
    })
}
