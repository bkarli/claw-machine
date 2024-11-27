claw-machine
============

## Parts list
### Gantry
| Name                  | Product                                                                                                      | Description                                                         | Price   |
|-----------------------|--------------------------------------------------------------------------------------------------------------|---------------------------------------------------------------------|---------|
| Nema 17 stepper Motor | [link](https://www.bastelgarage.ch/42x42x40mm-schrittmotor-nema-17-sm24240-1-7a-0-45nm?search=42x42x40)      | Motor to power X,Y,Z movement                                       | 3x15,90 |
| A4988 Driver          | [link](https://www.bastelgarage.ch/a4988-schrittmotor-treiber-stepper-driver-modul)                          | Driver to control the stepper motor (Direction and sleep state)     | 3x3,90  |
| Power supply          | [link](https://www.bastelgarage.ch/creality-cms-350-24-netzteil-24v-14-6a?search=netzteil%2024v)             | The PSU that powers the stepper motors                              | 1x59,90 |
| Linear guide          | [link](CNCMANS 4Stück 500mm Linearwelle Ø8mm Linearführung 500mm Linearführungen Präzisionswelle mit 8Stück) | Linear Guide in X and Y direction with bearings                     | 1x99,00 |
| Decoupling Capacitor  | ?                                                                                                            | Decoupling capacitors to secure Driver from voltage inconsistencies | ?       |

To move the carriages we will use rubber belts

### UI
| Name          | Product                                                                                    | Description                                                 | Price   |
|---------------|--------------------------------------------------------------------------------------------|-------------------------------------------------------------|---------|
| Joystick      | [link](https://www.bastelgarage.ch/arcade-joystick-4-weg)                                  | A joystick to control X,Y axis                              | 1x16,90 |
| Arcade Button | [link](https://www.bastelgarage.ch/arcade-taster-button-beleuchtet-60mm-rot?search=arcade) | Buttons to start the game and to start the grabbing process | 2x6,90  |

### Micro controller
For the project we will use an [Arduino Mega 2560](https://www.bastelgarage.ch/arduino-mega-2560-rev3) 

## Build Instructions
1. Install prerequisites as described in the [`avr-hal` README] (`avr-gcc`, `avr-libc`, `avrdude`, [`ravedude`]).

2. Run `cargo build` to build the firmware.

3. Run `cargo run` to flash the firmware to a connected board.  If `ravedude`
   fails to detect your board, check its documentation at
   <https://crates.io/crates/ravedude>.

4. `ravedude` will open a console session after flashing where you can interact
   with the UART console of your board.

[`avr-hal` README]: https://github.com/Rahix/avr-hal#readme
[`ravedude`]: https://crates.io/crates/ravedude

