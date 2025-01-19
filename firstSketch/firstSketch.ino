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
AccelStepper stepperYOne(AccelStepper::DRIVER, 14, 15);
AccelStepper stepperYTwo(AccelStepper::DRIVER, 18, 19);
AccelStepper stepperX(AccelStepper::DRIVER, 22, 23);
AccelStepper stepperSeil(AccelStepper::DRIVER, 26, 27);
Servo servoClaw;

//quick access to steppers though arrays
AccelStepper* steppersXY[] = {&stepperYOne, &stepperYTwo, &stepperX};
AccelStepper* allSteppers[] = {&stepperYOne, &stepperYTwo, &stepperX, &stepperSeil};

//more pins
const int limitSwitchX = 12;
const int limitSwitchY = 13;

const int stepperSpeed = 400;
int state;

//placeholders
signed int maxY = 1600;
signed int maxX = -1600;
int destinationX;
int destinationY;
int servoPos = 0;

void setup() {
  setPins();
  setStepperSpeeds(200);
  servoClaw.attach(7);
  state = States::RETURNING;
}

void loop() {
  //read input buttons
  int startButton = digitalRead(buttonPinStart);
  int endButton = digitalRead(buttonPinEnd);
  destinationY = stepperYOne.currentPosition();
  destinationX = stepperX.currentPosition();
  checkForEmergency();

  //if state is idle make sure steppers finish moving to idle state
  switch (state){
    case 0: //IDLE
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
        for (AccelStepper* runningStepper : steppersXY){
          runningStepper -> runSpeed();
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
        stepperX.setCurrentPosition(0);
        moveClawX(0);
      } else {
        moveClawX(100);
      }
      if (varY == HIGH){
        stepperYOne.stop();
        stepperYTwo.stop();
        stepperYOne.setCurrentPosition(0);
        stepperYTwo.setCurrentPosition(0);
        moveClawY(0);
      } else {
        moveClawY(-100);
      }
      //let all the x and y steppers move
      for (AccelStepper* runningStepper : steppersXY){
        runningStepper -> runSpeed();
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
void setStepperSpeeds(int speed){
  stepperSeil.setAcceleration(speed);
  stepperSeil.setMaxSpeed(200);
  for (AccelStepper* thisStepper : steppersXY){
    thisStepper -> setSpeed(0);
  }
}

//reading the joystick inputs and adding to the destination x and y
void readJoystick(){
  moveClawY(0);
  moveClawX(0);
  if (digitalRead(joyPinForward) == HIGH && digitalRead(joyPinBackward) == LOW){
    if (destinationY < maxY){
      moveClawY(stepperSpeed);
    }
  } else if (digitalRead(joyPinBackward) == HIGH && digitalRead(joyPinForward) == LOW){
    if (destinationY > 0){
      moveClawY(-stepperSpeed);
    }
  } else if (digitalRead(joyPinLeft) == HIGH && digitalRead(joyPinRight) == LOW){
    if (destinationX < 0){
      moveClawX(stepperSpeed);
    }
  } else if (digitalRead(joyPinRight) == HIGH && digitalRead(joyPinLeft) == LOW){
    if (destinationX > maxX){
      moveClawX(-stepperSpeed);
    }
  }
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
    stepperYOne.setSpeed(value);
    stepperYTwo.setSpeed(value);
  } else {
    stepperYOne.setSpeed(0);
    stepperYTwo.setSpeed(0);
  }
}

// same for the x stepper
void moveClawX(signed int value){
  if (0 >= destinationX  && destinationX >= maxX){
    stepperX.setSpeed(value);
  } else {
    stepperX.setSpeed(0);
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
  return ((servoPos == 0) || (servoPos == 45));
}

bool ropeFinished(){
  return !(stepperSeil.isRunning());
}
//checking if the x and y motors are finished moving
bool runningMotorsFinished(){
  bool stillToGo = false;
  for (AccelStepper* stepper : steppersXY){
    stillToGo = stillToGo || stepper -> isRunning();
  }
  return !stillToGo;
}

//dropping the claw and closing it, then lifting it
void grabItemInit(){
  dropClaw();
}

//dropping the claw
void dropClaw(){
  stepperSeil.moveTo(800);
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
    delay(44);
  }
}

//closing the claw
void closeClaw(){
  for (servoPos ; servoPos <= 45; servoPos += 1){
    servoClaw.write(servoPos);
    delay(44);
  }
}

//checking for emergency
void checkForEmergency(){
  if ((digitalRead(limitSwitchX) == HIGH || digitalRead(limitSwitchY) == HIGH) && state == RUNNING ){
    Serial.println("Limit switch triggered!");
    stopExecution();
  } else if (destinationX < maxX || destinationY > maxY){
    Serial.println("Out of bounds movement!");
    stopExecution();
  }
}

//in case of emergency!
void stopExecution() {
  while (true) {
    delay(1000);
  }
}