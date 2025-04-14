example-program-alarm-clock
===========================


This repo contains a simple Alarm Clock example in embedded rust for the Arduino Uno.
Through leveraging existing crates, we are able to compile Rust into the avr instruction set used by Arduino boards.

This simple program turn an "alarm" (speaker) on after approximately 7s. The user can snooze the alarm for 7s by clicking a button.

The button is connected to pin d2.
The speaker input is connected to pin d5.

### Acknowledgements
This code was derived using multiple examples from [this](https://github.com/Rahix/avr-hal/blob/main/examples/arduino-uno/src/bin/) repository -- thanks!



Rust project for the _Arduino Uno_.

## Build Instructions for Testing Outside of Iron Coder
1. Install prerequisites as described in the [`avr-hal` README] (`avr-gcc`, `avrdude`, [`ravedude`]).

2. Run `cargo build` to build the firmware.

3. Run `cargo run` to flash the firmware to a connected board.  If `ravedude`
   fails to detect your board, check its documentation at
   <https://crates.io/crates/ravedude>.

4. `ravedude` will open a console session after flashing where you can interact
   with the UART console of your board.

[`avr-hal` README]: https://github.com/Rahix/avr-hal#readme
[`ravedude`]: https://crates.io/crates/ravedude

## License
Licensed under either of

 - Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
 - MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

