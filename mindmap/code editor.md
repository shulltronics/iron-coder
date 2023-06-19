*this note is for thinking about the **code editor** portion of the project*

### Prior Art
[Lapce](https://github.com/lapce/lapce) seems really cool.
* I wonder though: does it expose a crate that I could integrate into my [[egui integrations]]? 
* Could I make the [[spec viewer]] be a separate crate that could also be compiled to WASI format as a Lapce plugin?

**TODO**
* __App__
  * Properly support colorschemes
  * Improve terminal integration
  * Provide tests to ensure that cargo, rustc, etc are properly installed on the host system
  * Provide a simple "status" message with single-line status updates based on latest actions
  * Integrate a proper debug logging system, in idiomatic Rust style
  * Integrate tests and benchmarks in idiomatic Rust style
  * Long-term goal is to have a "systems level" IDE, that might include multiple sub-projects. i.e. maybe one has a LoRa system that involves a gateway and multiple nodes, which would have two Programmable Boards. It would be awesome to have a way to help make the sub Projects consistent, such as shared protocol definitions, etc, and a way to test/benchmark the system as a whole
* __Project__
  * Read from/write to disk for save/open of projects
    * TODO -- check that we don't overwrite an existing project!
  * Allow for creation of new projects and editing of existing ones
    * Fix issue where CodeEditor state is lost after project edit
  * Force a project to make sense, i.e. should there be exactly one "Programmable Board"? Should I ensure that peripheral boards are compatible, or at least provide a warning if they might not be (i.e. using an I2S mic with a processor that doesn't have I2S)? A long-term goal is to be able to define the connections between boards (like I2C, SPI, etc), and like it to their pinouts.
  * Another long-term goal is to have "code blocks" that help link Programmable Boards to Peripheral Boards
  * Provide build stats like executable size, etc
  * Consider ways to measure and monitor performance. I'd like to write some benchmarking code similar to CoreMark, but in Rust. It would also be cool to benchmark other interesting HW-specific things, such as IO, floating-point stuff, bandwidth tests of WiFi/BLE/LoRa type boards.
* __Board__
  * Make Boards widget collapsible
  * Link docs.rs, datasheet, other resources to Board
  * Link examples to Board
    * ?? How to differentiate an "example" for a processor, vs an "example" project? Maybe for each Programmable Board there are core examples, and then there are also Project examples that use that board (possibly with other peripheral Boards). It might make sense either way for these to be packaged as Projects, likely read-only, similar to Arduino examples.
* __CodeEditor__
  * Line numbers (togglable in settings)
  * Fix issues with opening the same file twice, and reloading of file if edited elsewhere (or by Iron Coder itself, e.g. when editing the Project).
  * Syntax Highlighting
    * Benchmarking / optimizations
    * supporting all colorschemes of the overall app