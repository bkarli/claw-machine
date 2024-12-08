#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod timer;
mod button;
mod limit_switch;
mod joystick;
mod game;

use panic_halt as _;
use crate::timer::{GenericTicker, PrecisionTicker};

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
    // create the game
    let game = game::Game::new();

    // enable interrupts for the device
    unsafe { avr_device::interrupt::enable() };

    game.run()

}
