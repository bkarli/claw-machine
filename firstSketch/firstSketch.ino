//need to have AccelStepper downloaded in arduinoIDE as dependency!
#include <AccelStepper.h>
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
AccelStepper stepperClaw(AccelStepper::FULL4WIRE, 30, 31, 32, 33);

//quick access to steppers though arrays
AccelStepper steppersXY[] = {stepperYOne, stepperYTwo, stepperX};
AccelStepper steppersClawSeil[] = {stepperClaw, stepperSeil};

//more pins
const int limitSwitchX = 12;
const int limitSwitchY = 13;

int startButton;
int endButton;
int state;

int maxY = 1000;
int maxX = 1000;
int destinationX;
int destinationY;


void setup() {
  setPins();
  setStepperSpeeds(20, 5);
  state = States::IDLE;
}

void loop() {
  // check start and end buttons, check state and modify state accordingly
  // if state = idle, check position and claw, if not correct move to correct position and open claw
  // if state = running, check moving pins inputs
  //    move claw
  startButton = digitalRead(buttonPinStart);
  endButton = digitalRead(buttonPinEnd);
  if (checkState() == IDLE){
    for (AccelStepper idleStepper : steppersClawSeil){
      idleStepper.run();
    }
    if (startButton == HIGH && idleMotorsFinished()){
      startGame();
    }
  }else if (checkState() == RUNNING){
    if (endButton == HIGH && runningMotorsFinished()){
      endGame();
    } else {
      readJoystick();
      for (AccelStepper runningStepper : steppersXY){
        runningStepper.run();
      }
    }
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

void setStepperSpeeds(int speed, int acceleration){
  for (AccelStepper thisStepper : steppersXY){
    thisStepper.setSpeed(speed);
    thisStepper.setAcceleration(acceleration);
  }
  for (AccelStepper thisStepper : steppersClawSeil){
    thisStepper.setSpeed(speed);
    thisStepper.setAcceleration(acceleration);
  }
}

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

int checkState(){
  return state;
}
void changeState(States toBeState){
  state = toBeState;
}

void startGame(){
  changeState(RUNNING);
}

void moveClawY(signed int value){
  if (0 <= destinationY  && destinationY <= maxY){
    stepperYOne.move(value);
    stepperYTwo.move(value);
  }
}
void moveClawX(signed int value){
  if (0 <= destinationX  && destinationX <= maxX){
    stepperX.move(value);
  }
}

void moveClawToIdle(){
  int isLimitY = digitalRead(limitSwitchY);
  while (isLimitY == LOW){
    if (digitalRead(limitSwitchY) == HIGH){
      stepperYOne.stop();
      stepperYTwo.stop();
      isLimitY = HIGH;
      return;
    }
    moveClawY(1);
  }
  int isLimitX = digitalRead(limitSwitchX);
  while (isLimitX == LOW){
    if (digitalRead(limitSwitchX) == HIGH){
      stepperX.stop();
      return;
    }
    moveClawX(1);
  }
  destinationX = 0;
  destinationY = 0;
  openClaw();
}

void endGame(){
  changeState(IDLE);
  moveClawToIdle();
}

bool idleMotorsFinished(){
  return !(stepperClaw.isRunning() || stepperSeil.isRunning());
}

bool runningMotorsFinished(){
  bool stillToGo = false;
  for (AccelStepper stepper : steppersXY){
    stillToGo = stillToGo || stepper.isRunning();
  }
  return !stillToGo;
}
void dropClaw(){
  //drop it (to implement), !!after!!
  closeClaw();
  liftClaw();
}
void liftClaw(){}
void openClaw(){}
void closeClaw(){}
