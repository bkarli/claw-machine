use core::sync::atomic::AtomicBool;
use arduino_hal::hal::port::Dynamic;
use arduino_hal::port::mode::{Input, PullUp};
use arduino_hal::port::Pin;
use crate::button::{Button, ButtonType};

/**
All possible game states
idle => the game has reset and is ready for a new round
running => one is currently playing the game
finished => one has finished tha game and machine resets
*/

pub enum GameState {
    IDLE,
    RUNNING,
    FINISHED
}

/**
struct for the game and its logic
*/
pub(crate) struct Game {
    state: GameState,
}

impl Game {
    pub fn new() -> Self {
        Self {
            state: GameState::IDLE
        }
    }

    /**
    The main game loop

    Controls the program flow

    RETURNS: ! (never returns)
    */
    pub fn run(
        &mut self,
        start_button_pin: Pin<Input<PullUp>,Dynamic>
    ) -> ! {
        let start_button: Button = Button::new(ButtonType::GameStart, start_button_pin);
        loop {
            match self.state {
                GameState::IDLE => {
                    // wait for start button
                    self.idle_game(&start_button);
                }
                GameState::RUNNING => {

                }
                GameState::FINISHED => {
                    self.reset_game();
                }
            }
        }
    }

    async fn run_game(&mut self) {

    }

    async fn idle_game(&mut self, start_button: &Button) {
        loop {
            // await press of start button

            self.state = GameState::RUNNING;
        }
    }

    async fn reset_game(&mut self) {
        // move pulley up
        // move carriage left
        // move carriage forward
        self.state = GameState::IDLE;
    }
}

