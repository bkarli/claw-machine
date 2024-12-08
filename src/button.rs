use arduino_hal::hal::port::Dynamic;
use arduino_hal::port::mode::{Input, PullUp};
use arduino_hal::port::Pin;

/**
Enum for All possible UI Buttons that a User can press

GameStart => Starts the game
GameFinish => Ends the game
*/
pub enum ButtonType {
    GameStart,
    GameFinish
}

/**
struct for the UI buttons
*/
pub struct Button {
    button_type: ButtonType,
    pin: Pin<Input<PullUp>, Dynamic>,
}

impl Button {
    pub fn new(
        button_type: ButtonType,
        pin: Pin<Input<PullUp>>,
    ) -> Self {
        Self {
            button_type,
            pin
        }
    }
}
