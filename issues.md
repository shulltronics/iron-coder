### Known Issues

1. **Settings Issues**: The font selector and adding a GitHub account do not function.
2. **OLED Featherwing Board Position**: Occasionally, adding an OLED Featherwing board places it at the bottom left, making it difficult to locate. *Minor issue*.
3. **OLED Featherwing Pins**: The pins on the OLED Featherwing board do not light up.
4. **PiTFT 3.2 Pins**: For the PiTFT 3.2 with Capacitive Touch Screen, only the middle pins light up.
5. **Save Project**: The project must be saved before development can start.
6. **Nightly Channel Requirement**: To build the project and load it onto the board, switching to the Nightly Channel and using the `-Z unstable-options` flag is required.
7. **Cargo Generate Package**: To use the `gen template` and `gen sys mod` options, the `cargo-generate` package must be installed."