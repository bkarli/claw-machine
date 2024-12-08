# Technical Documentation for the Claw Machine

## Goals

## Software Design

### Interrupts
We are only using Interrupts for Input devices, which latency and thus improving the responsiveness
of our input devices. As our Microcontroller only allows for 6 directly interruptible Pins. Which are
INT0 (Pin 2) ,INT1 (Pin 3), INT3 (Pin X), INT4 (Pin X), INT5 (Pin X), INT6 (Pin X), INT7 (Pin X) (source). As we have
8 Input devices we either have to choose 6 Devices to use for interrupts and busy wait for the other two. And
depending on the devices the performance hit would have been negligible. But there is another way. 
Instead of listening to selected pin the AVRMega2560 chips allows for Pin Change interrupt on selected Ports

These interrupts are called PCINTn and are available for Port X, Port Y and Port Z. (source) So we would have
three interrupts to play with, which would suit our project perfectly as we have three different encapsulated
systems that are listening for inputs at different states of our Program.

In detail, we would have one port dedicated for our limit switches which will only be active when the machine
enters reset mode, and its carriage as well as the pulley returns to its initial state.
Another port is dedicated for our Joystick, which internally are four buttons, and are only active
when a player is currently playing the game.
And the last Interrupt would be dedicated to our Game Input buttons will start the game and end the game.

Using this approach we are able to disable interrupts when the current Input device is not in use and thus
reducing load on the system. The extra work for the system will be, that it needs to figure out which pin caused
the interrupt once it happened. 

### Timers

#### Timer Events
Instead of busy waiting for Timer to run out, each Timer provided by the Microcontroller has two Interrupts to use,
which depending on the use case can be used to trigger software side interrupts that we can use.

1. Timer Overflow Event
2. Timer Compare Match

The Timer Overflow Event is as the name suggests triggered when the counter register of the respective Timer overflows
depending on the prescaler and the register bit length can be in different precisions and is often used to get an 
accurate reading on how much time has passed since the program has started.

Timer Compare Match lets us load a number, the length is depending on the register length of the counter itself, into 
another register and at each increment it compares the loaded number with the counter and triggers an interrupt once it
is matches. We will use these Compare Match Events to trigger our Timer Events.

#### Available Timers
Different parts of our system need different time specific accuracies and thus need different timers.
Looking at the AVRmega2560 datasheet we have 5 different timer registers to play with. We will be using three of them

1. Timer0 has a counter register with length of 8 bits, meaning it will overflow when it reaches 255 and has PWM support
2. Timer1 has a counter register with length of 16 bits, meaning it will overflow when it reaches 65535
3. Timer2 is similar to the Timer0 and also in terms of counter register length and also PWM support

Timer0 will be used to create Timer Events in the millisecond range to control the speed of our stepper motors
as they need pulses in that range to accelerate and decelerate. 

Timer1 will be used to create Timer Events in the seconds range to control certain actions of the Game for dramatic 
effects, like close the claw after 1 second once the pulley is finished of letting the claw down. Or limit the play time
for the player to a certain amount, and if he hasn't finished the game in time the system will finish the game for the 
player.

Timer2 will be used for PWM for the claw servo motor. By using the simplePWM abstraction provided by the HAL.

### Idle state
The carriage as well as the pulley of our machine is powered by powerful stepper motors. As they draw a lot of power
and when they draw power strain not only the stepper motor driver but the motor itself. Ideally they need to be in sleep
mode when not in use. This can be achieved by pulling the sleep pin high on the stepper motor driver.

### Async Await
With all Input devices linked to Interrupts and all timing specific events are also triggered by interrupts our system 
spends most of it's time sleeping and is only doing something when an Event has been triggered which is exactly what we
wanted. This leads us to the question, how do we design our system to handle theses asynchronous events safely and
efficiently. 

