

/**
All possible game states
idle => the game has reset and is ready for a new round
running => one is currently playing the game
finished => one has finished tha game and machine resets
*/
enum GameState {
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
    pub fn run(&self) -> ! {
        loop {

        }
    }

}