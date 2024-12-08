use core::cell::RefCell;
use avr_device::atmega2560::{TC0, TC1};
use heapless::binary_heap::{BinaryHeap, Min};

// Type alias for avr_decvice::interrupt::Mutex to Mutex
type Mutex<T> = avr_device::interrupt::Mutex<T>;



/// declare static precision ticker
static P_TICKER: PrecisionTicker = PrecisionTicker{ tc0: Mutex::new(RefCell::new(None)), max: 250 };
/// declare static generic Ticker
static G_TICKER: GenericTicker = GenericTicker {tc1: Mutex::new(RefCell::new(None)), max: 62500 };

/// binary heap that acts as the priority queue for our Precision timers

static P_QUEUE: Mutex<RefCell<BinaryHeap<(u16, usize), Min, 8>>> = Mutex::new(RefCell::new(BinaryHeap::new()));

/// binary heap that acts as the priority queue for our Generic timers

static G_QUEUE: Mutex<RefCell<BinaryHeap<(u32, usize), Min, 4>>> = Mutex::new(RefCell::new(BinaryHeap::new()));

/**
Constant conversion that convert microseconds to precision ticks

250 ticks equals one millisecond

Min is 1 Tick which is 2.5 microseconds

*/
const fn us_to_p_ticks(us: u16) -> u16 {
    250 * us / 1000
}

/**
    Constant conversion that convert seconds to generic ticks
    Max register: 65535
    Our max 62'500
    When our max match occurs every four seconds
*/
const fn s_to_generic_ticks(s: u8) -> u32 {
    62500 * s as u32 / 4
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
        // tc1.ocr1a.write(|w|w.bits(62500));
        // set flag to only count to max and set CTC mode
        tc1.tccr1a.write(|w| w.wgm1().bits(4));
        // set prescale to 1024
        tc1.tccr1b.write(|w| w.cs1().prescale_1024());
        // enable CTC mode interrupt
        tc1.timsk1.write(|w| w.ocie1a().set_bit());

        // replace the tc1
        avr_device::interrupt::free(|cs| {
            G_TICKER.tc1.borrow(cs).replace(Some(tc1));
        })
    }
}
/**
Ticker to generate Ticker with microsecond precision used internally to generate Timer Events with
microsecond precision.

we will be using delays between 300..3000 microseconds to accelerate and decelerate our motors, so
ideally we use some sort of combination of prescaler and register value that overflows or triggers
every 100 microseconds

with a prescaler of 64 and a max value we know that the timer will at least overflow every ms

*/
pub struct PrecisionTicker {
    pub(crate) tc0: Mutex<RefCell<Option<TC0>>>,
    pub max: u8
}

impl PrecisionTicker {
    pub fn init(tc0: TC0) {
        // enable CTC (clear timer on compare match)
        tc0.tccr0a.write(|w| w.wgm0().ctc());
        // choose the prescaler of the counter register
        tc0.tccr0b.write(|w| w.cs0().prescale_64());

        // replace tc0
        avr_device::interrupt::free(|cs| {
            P_TICKER.tc0.borrow(cs).replace(Some(tc0));
        })
    }
}

struct PrecisionTimer {
    ticks_till_end: u16
}

impl PrecisionTimer {
    pub fn new(microseconds: u16) -> Self {
        Self {
            ticks_till_end: us_to_p_ticks(microseconds)
        }
    }

    fn register(&self, task: usize) {
        avr_device::interrupt::free(|cs| {
            let mut queue = P_QUEUE.borrow(cs).borrow_mut();
            let first = if let Some((first,_)) = queue.peek() {
                self.ticks_till_end < *first
            }else { true };
            if queue.push((self.ticks_till_end, task)).is_err() {
                panic!("Queue is full");
            }
            if first {
                schedule_precision_wakeup()
            }
        })
    }
}

struct GenericTimer {
    ticks_till_end: u32
}

impl GenericTimer {
    pub fn new(seconds: u8) -> Self {
        Self {
            ticks_till_end: s_to_generic_ticks(seconds)
        }
    }
}

/**
Add a new timer to the precision queue
*/
fn schedule_precision_wakeup() {

}

/**
Add a new timer to the generic queue
*/
fn schedule_generic_wakeup() {

}



/**
Interrupt triggered at least every 1 millisecond
*/
#[avr_device::interrupt(atmega2560)]
fn TIMER0_COMPA() {

}


/**
Interrupt triggered at least every 4 seconds
*/
#[avr_device::interrupt(atmega2560)]
fn TIMER1_COMPA() {

}