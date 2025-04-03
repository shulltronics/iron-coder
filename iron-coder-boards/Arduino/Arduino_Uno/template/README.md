Embedded Rust Template Project for Arduino Uno
=================================================

This repo contains a simple blinky-led example in embedded rust for the Arduino Uno. 
Through leveraging existing crates, we are able to compile Rust into the avr instruction set used by Arduino boards.

### Acknowledgements
Most of this code is adapted from [this](https://github.com/Rahix/avr-hal/blob/main/examples/arduino-uno/src/bin/uno-blink.rs) repository -- thanks!




## Build Instructions used to Test Code Fuctionality Outside of Iron Coder
1. Install prerequisites as described in the [`avr-hal` README] (`avr-gcc`, `avr-libc`, `avrdude`, [`ravedude`]).

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
