//need to have AccelStepper & Servo downloaded in arduinoIDE as dependency!
#include <AccelStepper.h>
#include <Servo.h>
#include "firstSketch.h"

//setting pins
const int lampRed = 9;
const int lampGreen = 10;
const int joyPinLeft = 51;
const int joyPinRight = 53;
const int joyPinForward = 50;
const int joyPinBackward = 52;
const int buttonPinStart = 15;
const int buttonPinEnd = 14;

//making steppers
AccelStepper stepperYOne(AccelStepper::DRIVER, 23, 22);
AccelStepper stepperYTwo(AccelStepper::DRIVER, 25, 24);
AccelStepper stepperX(AccelStepper::DRIVER, 27, 26);
AccelStepper stepperSeil(AccelStepper::DRIVER, 31, 30);
Servo servoClaw;

//quick access to steppers though arrays
AccelStepper* steppersXY[] = {&stepperYOne, &stepperYTwo, &stepperX};
AccelStepper* allSteppers[] = {&stepperYOne, &stepperYTwo, &stepperX, &stepperSeil};

const int stepperSpeed = 600;
int state;

//placeholders
signed int maxY = 1600;
signed int maxX = 1600;
int destinationX;
int destinationY;
signed int servoPos = 50;

void setup() {
  setPins();
  setStepperSpeeds(0);
  servoClaw.attach(5);
  stepperSeil.moveTo(-1200);
  openClaw();
  state = States::IDLE;
  Serial.begin(9600);
  stepperX.setCurrentPosition(0);
  stepperYOne.setCurrentPosition(0);
  stepperYTwo.setCurrentPosition(0);
}

void loop() {
  int endButton = digitalRead(buttonPinEnd);
  int startButton = digitalRead(buttonPinStart);
  destinationY = stepperYOne.currentPosition();
  destinationX = stepperX.currentPosition();
  for (AccelStepper* runningStepper : steppersXY){
    runningStepper -> runSpeed();
  }
  switch (state){
    case IDLE: //IDLE
      digitalWrite(lampGreen, HIGH);
      stepperSeil.run();
      //if they are finished moving and start button is pressed switch state
      if (startButton == LOW && ropeFinished()){
        Serial.println("game start");
        digitalWrite(lampGreen, LOW);
        digitalWrite(lampRed, HIGH);
        startGame();
      }
      break;

    case RUNNING: //RUNNING
      if (endButton == LOW){
        setStepperSpeeds(0);
        endGame();
      } else {
        readJoystick();
      }
      break;

    case DROPPING: //DROPPING
      stepperSeil.run();
      if (ropeFinished()){
        Serial.println("rope finished");
        closeClaw();
        liftClaw();
      }
      break;

    case LIFTING: //LIFTING
      stepperSeil.run();
      if (ropeFinished()){
          moveClawToIdle();
        }
      break;

    case RETURNING: //RETURNING
      bool one = false;
      bool two = false;
      speedClawY(-100);
      speedClawX(-100);
      if ( 0 == destinationX){
        speedClawX(0);
        one = true;
      }
      if ( 0 == destinationY){
        speedClawY(0);
        two = true;
      }
      if (one && two ){
        digitalWrite(lampRed, LOW);
        digitalWrite(lampGreen, HIGH);
        openClaw();
        changeState(IDLE);
      }
      break;
  }
}

// setting the pins to in- / output
void setPins(){
  pinMode(lampRed, OUTPUT);
  pinMode(lampGreen, OUTPUT);
  pinMode(joyPinLeft, INPUT_PULLUP);
  pinMode(joyPinRight, INPUT_PULLUP);
  pinMode(joyPinForward, INPUT_PULLUP);
  pinMode(joyPinBackward, INPUT_PULLUP);

  pinMode(buttonPinStart, INPUT_PULLUP);
  pinMode(buttonPinEnd, INPUT_PULLUP);
}

//setting stepper speeds with placeholder speeds
void setStepperSpeeds(int speed){
  stepperSeil.setAcceleration(800);
  stepperSeil.setMaxSpeed(400);
  stepperSeil.setSpeed(200);
  for (AccelStepper* thisStepper : steppersXY){
    thisStepper -> setMaxSpeed(600);
    thisStepper -> setAcceleration(1000);
    thisStepper -> setSpeed(speed);
  }
}

//reading the joystick inputs and adding to the destination x and y
void readJoystick(){
  if (digitalRead(joyPinForward) == LOW && digitalRead(joyPinBackward) == HIGH){
    if (destinationY <= maxY){
      speedClawY(stepperSpeed);
      speedClawX(0);
    } else {
      speedClawY(-stepperSpeed);
      speedClawX(0);
    }
  } else if (digitalRead(joyPinBackward) == LOW && digitalRead(joyPinForward) == HIGH){
    if (0 <= destinationY){
      speedClawY(-stepperSpeed);
      speedClawX(0);
    }else {
      speedClawY(stepperSpeed);
      speedClawX(0);
    }
  } else if (digitalRead(joyPinLeft) == LOW && digitalRead(joyPinRight) == HIGH){
    if (destinationX >= maxX){
      speedClawX(-stepperSpeed);
      speedClawY(0);
    }else {
      speedClawX(-stepperSpeed);
      speedClawY(0);
    }
  } else if (digitalRead(joyPinRight) == LOW && digitalRead(joyPinLeft) == HIGH){
    if (destinationX <= 0){
      speedClawX(stepperSpeed);
      speedClawY(0);
    } else {
      speedClawX(stepperSpeed);
      speedClawY(0);
    }
  } else {
    speedClawX(0);
    speedClawY(0);
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
void speedClawY(signed int value){
  stepperYOne.setSpeed(value);
  stepperYTwo.setSpeed(value);
}

// same for the x stepper
void speedClawX(signed int value){
  stepperX.setSpeed(value);
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
  return ((servoPos == -10) || (servoPos == 50));
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
  stepperSeil.moveTo(0);
  changeState(DROPPING);
}

//lifting the claw
void liftClaw(){
  stepperSeil.moveTo(-1200);
  changeState(LIFTING);
}

//opening the claw
void openClaw(){
  for (servoPos; servoPos >= -10; servoPos -= 1){
    servoClaw.write(servoPos);
    delay(20);
  }
}

//closing the claw
void closeClaw(){
  for (servoPos ; servoPos <= 50; servoPos += 1){
    servoClaw.write(servoPos);
    delay(20);
  }
}

//in case of emergency!
void stopExecution() {
  while (true) {
    delay(1000);
  }
}