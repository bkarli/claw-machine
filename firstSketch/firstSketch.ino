//need to have AccelStepper & Servo downloaded in arduinoIDE as dependency!
#include <AccelStepper.h>
#include <Servo.h>
#include "firstSketch.h"

//setting pins
const int joyPinLeft = 1;
const int joyPinRight = 2;
const int joyPinForward = 3;
const int joyPinBackward = 4;
const int buttonPinStart = 5;
const int buttonPinEnd = 6;

//making steppers
AccelStepper stepperYOne(AccelStepper::FULL4WIRE, 14, 15, 16, 17);
AccelStepper stepperYTwo(AccelStepper::FULL4WIRE, 18, 19, 20, 21);
AccelStepper stepperX(AccelStepper::FULL4WIRE, 22, 23, 24, 25);
AccelStepper stepperSeil(AccelStepper::FULL4WIRE, 26, 27, 28, 29);
Servo servoClaw;

//quick access to steppers though arrays
AccelStepper steppersXY[] = {stepperYOne, stepperYTwo, stepperX};
AccelStepper allSteppers[] = {stepperYOne, stepperYTwo, stepperX, stepperSeil};

//more pins
const int limitSwitchX = 12;
const int limitSwitchY = 13;

int startButton;
int endButton;
int state;

//placeholders
int maxY = 1000;
int maxX = 1000;
int destinationX;
int destinationY;
int servoPos = 0;


void setup() {
  setPins();
  setStepperSpeeds(20, 5);
  servoClaw.attach(7);
  state = States::RETURNING;

}

void loop() {
  //read input buttons
  startButton = digitalRead(buttonPinStart);
  endButton = digitalRead(buttonPinEnd);
  switch (state){
    case 0: //IDLE
      //let all steppers move if they have to
      for (AccelStepper stepper : allSteppers){
      stepper.run();
      }
      //if they are finished moving and start button is pressed switch state
      if (startButton == HIGH && clawAndRopeFinished()){
        startGame();
      }
      break;

    case 1: //RUNNING
      //check if the steppers are still and end button is pressed
      if (endButton == HIGH && runningMotorsFinished()){
        endGame();
      // read the joystick input and move the claw motors
      } else {
        readJoystick();
        //let all the x and y stepper move
        for (AccelStepper runningStepper : steppersXY){
          runningStepper.run();
        }
      }
      break;

    case 2: //DROPPING
      //let the rope stepper go to its destination
      stepperSeil.run();
      if (ropeFinished()){
        //this takes a fixed amount of time and delays all other things
        closeClaw();
        liftClaw();
      }
      break;

    case 3: //LIFTING
      //let the rope stepper finish moving
      stepperSeil.run();
      if (ropeFinished()){
        //return the claw to its original position
        moveClawToIdle();
      }
      break;

    case 4: //RETURNING
      int varX = digitalRead(limitSwitchX);
      int varY = digitalRead(limitSwitchY);
      if (varX == HIGH){
        stepperX.stop();
        destinationX = 0;
      } else {
        moveClawX(-1);
      }
      if (varY == HIGH){
        stepperYOne.stop();
        stepperYTwo.stop();
        destinationY = 0;
      } else {
        moveClawY(-1);
      }
      //let all the x and y steppers move
      for (AccelStepper runningStepper : steppersXY){
        runningStepper.run();
      }
      //if they have arrived, open the claw and go to idle state
      if (destinationX == 0 && destinationY == 0){
        openClaw();
        changeState(IDLE);
      }
      break;
  }
}

// setting the pins to in- / output
void setPins(){
  pinMode(joyPinLeft, INPUT);
  pinMode(joyPinRight, INPUT);
  pinMode(joyPinForward, INPUT);
  pinMode(joyPinBackward, INPUT);

  pinMode(buttonPinStart, INPUT);
  pinMode(buttonPinEnd, INPUT);

  pinMode(limitSwitchX, INPUT);
  pinMode(limitSwitchY, INPUT);
}

//setting stepper speeds with placeholder speeds
void setStepperSpeeds(int speed, int acceleration){
  for (AccelStepper thisStepper : allSteppers){
    thisStepper.setSpeed(speed);
    thisStepper.setAcceleration(acceleration);
  }
}

//reading the joystick inputs and adding to the destination x and y
void readJoystick(){
  if (digitalRead(joyPinForward) == HIGH){
    destinationY += 1;
    moveClawY(1);
  } else if (digitalRead(joyPinBackward) == HIGH){
    destinationY -= 1;
    moveClawY(-1);
  } else if (digitalRead(joyPinLeft) == HIGH) {
    destinationX += 1;
    moveClawX(1);
  } else if (digitalRead(joyPinRight) == HIGH){
    destinationX -= 1;
    moveClawX(-1);
  }
}

// reading the current state
int checkState(){
  return state;
}

// changing the current state
void changeState(States toBeState){
  state = toBeState;
}

//starting the game by changing to state running
void startGame(){
  changeState(RUNNING);
}

// moving the y steppers in parallel if destination is allowed
void moveClawY(signed int value){
  if (0 <= destinationY  && destinationY <= maxY){
    stepperYOne.move(value);
    stepperYTwo.move(value);
  }
}

// same for the x stepper
void moveClawX(signed int value){
  if (0 <= destinationX  && destinationX <= maxX){
    stepperX.move(value);
  }
}

//moving to the idle destination and opening the claw to drop the item
void moveClawToIdle(){
  changeState(RETURNING);
}

//ending the game by switching the state to idle and moving the claw to the idle position
void endGame(){
  grabItemInit();
}

//checking if the claw and rope motors are finished
bool clawAndRopeFinished(){
  return (clawFinished() && ropeFinished());
}

//unsure!
bool clawFinished(){
  return ((servoPos == 0) || (servoPos == 180));
}

bool ropeFinished(){
  return !(stepperSeil.isRunning());
}
//checking if the x and y motors are finished moving
bool runningMotorsFinished(){
  bool stillToGo = false;
  for (AccelStepper stepper : steppersXY){
    stillToGo = stillToGo || stepper.isRunning();
  }
  return !stillToGo;
}

//dropping the claw and closing it, then lifting it
void grabItemInit(){
  dropClaw();
}

//dropping the claw
void dropClaw(){
  stepperSeil.moveTo(500);
  changeState(DROPPING);
}

//lifting the claw
void liftClaw(){
  stepperSeil.moveTo(0);
  changeState(LIFTING);
}

//opening the claw
void openClaw(){
  for (servoPos; servoPos >= 0; servoPos -= 1){
    servoClaw.write(servoPos);
    delay(15);
  }
}

//closing the claw
void closeClaw(){
  for (servoPos ; servoPos <= 180; servoPos += 1){
    servoClaw.write(servoPos);
    delay(15);
  }
}
