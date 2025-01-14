
use core::pin::pin;
use arduino_hal::hal::port::{Dynamic, PB7, PE3};
use arduino_hal::port::mode::{Output, PwmOutput};
use arduino_hal::port::Pin;
use arduino_hal::simple_pwm::Timer3Pwm;
use avr_device::atmega2560::EXINT;
use avr_device::interrupt;
use crate::channel::{Channel, Receiver, Sender};
use crate::{executor, J_RIGHT};
use crate::executor::wake_task;
use crate::joystick::{joystick_switch_task, JoystickDirection};
use crate::stepper::{stepper_task_x, Stepper, StepperDirection};

/**
All possible game states
idle => machine resets and is ready for a new round
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
    exint: EXINT
}

impl Game {
    pub fn new(
        exint: EXINT,
    ) -> Self {
        Self {
            state: GameState::IDLE,
            exint
        }
    }

    /**
    The main game loop

    Controls the program flow

    RETURNS: ! (never returns)
    */
    pub fn run(
        &mut self,
        x_stepper_pulse: Pin<Output, Dynamic>,
        x_stepper_direction: Pin<Output, Dynamic>,
        y_stepper_pulse: Pin<Output, Dynamic>,
        y_stepper_direction: Pin<Output, Dynamic>,
        y_stepper_pulse_inverted: Pin<Output, Dynamic>,
        y_stepper_direction_inverted: Pin<Output, Dynamic>,
        z_stepper_pulse: Pin<Output, Dynamic>,
        z_stepper_direction: Pin<Output, Dynamic>,
        mut claw_pwm: Pin<PwmOutput<Timer3Pwm>, PE3>
    ) -> ! {
        loop {
            match self.state {
                GameState::IDLE => {
                    // before idle the games resets its physical state
                    // this will be run at the start of the program and also after each finished
                    // game

                    // enable limit switch interrupts
                    self.exint.pcicr.write(|w| unsafe {w.bits(0b100)});
                    self.exint.pcmsk2.write(|w| w.bits(0b00000111));

                    let reset_task = pin!(reset_game());
                    executor::run_task(&mut [reset_task]);
                    claw_pwm.enable();
                    claw_pwm.set_duty(255);



                    // enable UI button interrupts and disable limit switch interrupts
                    self.exint.pcicr.write(|w| unsafe {w.bits(0b010)});
                    self.exint.pcmsk1.write(|w| w.bits(0b00000010));

                    

                    // executor execute ui buttons task
                    executor::run_task(&mut []);

                    // once executor loop breaks change game state
                    self.state = GameState::RUNNING
                }
                GameState::RUNNING => {
                    // enable all interrupts except limit switches
                    self.exint.pcicr.write(|w| unsafe {w.bits(0b011)});
                    // Joystick pc interrupt pins
                    self.exint.pcmsk0.write(|w| w.bits(0b00001111));
                    // end button interrupt pin
                    self.exint.pcmsk1.write(|w| w.bits(0b00000100));

                    let x_channel: Channel<StepperDirection> = Channel::new();
                    let y_channel: Channel<StepperDirection> = Channel::new();

                    // create all joystick tasks
                    let joystick_right_task = pin!(joystick_switch_task(
                        JoystickDirection::RIGHT,
                        x_channel.get_sender()
                    ));
                    let joystick_left_task = pin!( joystick_switch_task(
                        JoystickDirection::LEFT,
                        x_channel.get_sender()
                    ));
                    let joystick_forward_task = pin!( joystick_switch_task(
                        JoystickDirection::FORWARD,
                        y_channel.get_sender()
                    ));
                    let joystick_backward_task = pin!( joystick_switch_task(
                        JoystickDirection::BACKWARD,
                        y_channel.get_sender()
                    ));

                    /*let x_axis_task = pin!(stepper_task_x(
                        x_stepper_pulse,
                        x_stepper_direction,
                        x_channel.get_receiver()
                    ));
                    */

                    executor::run_task(&mut [
                        joystick_right_task,
                        joystick_left_task,
                        joystick_forward_task,
                        joystick_backward_task
                    ]);

                    self.state = GameState::FINISHED
                }
                GameState::FINISHED => {
                    // disable all interrupts
                    self.exint.pcicr.write(|w| unsafe {w.bits(0b000)});
                    self.state = GameState::IDLE;

                    // move claw down and close claw
                    claw_pwm.set_duty(155);
                }
            }
        }
    }
}


async fn reset_game(){
    // rollback z motor till limit switch
    // rollback x motor till limit switch
    // rollback y motor till limit switch
    // release claw

    // break the executor loop to advance to idle state
    wake_task(0xFFFF);
}
