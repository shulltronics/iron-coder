# Iron Coder Work Done (Pre-Alpha Build)
  - Updated version of ra_ap_rust-analyzer to 0.0.237 to allow code to build
  - Fixed flagging error when building project by switching to nightly channel
  - Fixed red box bug when dragging elements in the hardware editor
  - Cloned and built project to a working state
  - Setup GitHub organization for project work
  - Architecture for example code built, can load examples from menu 
  - Architecture for automatically generating board planned out, including where the new menu button needs to be added in source code
  - Create structs to store persistent state for fields needed to automatically generate boards
  - Learning Rust, looking through EGUI documentation to help understand the existing code base
  - Research examples of possible IronCoder structure (Wokwi, Fritzing, Arduino)
  - Design ideation/wireframing for simulator and debugging modes
  - Iron Coder Forum work done is listed in the [forum repository](https://github.com/CAPSTONE-24-25-IRON-CODER/iron-coder-forum)
# Iron Coder Work Done (Prototype)
  - Gained a better understanding of the process for creating new components by adding various components through creating TOML files and SVG files
    - Added Arduino Uno as a main board
    - Added Adafruit's 8x8 NeoPixel as a peripheral board
    - Added an 1000 Ω Resistor
    - Added a Yellow LED
    - Added a push button
    - Added a piezo buzzer
  - Support the ability to run Rust code on the Arduino Uno
  - Example code feature finished
  - Added Rust Examples to example folder to be loaded within Iron Coder
      - Alarm Clock Example (Arduino Uno)
      - LCD Screen Example
      - Blink LED Example
      - LED Array Example
  - Updated board menu
      - Changed name of board menu to component menu
      - Updated headers of each component to display board type (Main, Peripheral, Discrete)
      - Sorted components in the menu by board type and name
  - Updated Board struct to support discrete components as a type
      - Created new BoardType enum
      - Change is_main boolean field in Board struct to a BoardType enum
      - Implemented Ord trait for Board (allows us to sort by board type and name)
      - Change internal storage of components by adding a discrete components vector. When a component is added to the hardware editor, it is either added to the peripheral device vector, discrete components vector, or the main board variable.
      - Update .ironcoder TOML files for Example Code hardware editor state to support new discrete component type and vector
  - Create initial module for serial monitor debugger
      - Created data struct to contain serial data
      - Used egui to start testing with plotter
      - Connected example program to serial monitor console on VSCode
      - Researched how to connect code variables to serial
  - Can run Rust programs on Renode supported boards
      - Created an example program to utilize USART on STM32F4 Discovery board
      - Made a Renode script to load board and program .elf file
      - Researched information on how to support new boards such as the AdaFruit Feather RP2040
      - Planned a path to support the AdaFruit Feather RP2040 by production

# Iron Coder Architecture
Iron Coder is split into 3 main crates, one that handels the application itself, one for the boards, and the last for projects. The application uses both the boards and projects to
seemlessly create both the hardware and code editors. Within the project source code the project serves as a container for the path to where the project is stored, the boards that
are being used, functions to load/save projects, and anything else the application would need from it during run time. The boards serve a similar purpose in storing the functions and
parameters that are needed to describe the boards and work within the hardware editor. 
# Known bugs
  - Window stays open after loading example, need research into egui to figure out how to close after clicking example
  - Sometimes drawn wire connections will appear on top of the add board menu
  - More than one discrete component cannot be added to the hardware editor
  - Rx pin in STM32 board example does not work properly so we can only output data

