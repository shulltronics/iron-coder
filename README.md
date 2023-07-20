# Iron Coder
A system-level, hardware-aware embedded Rust IDE.

#### By Carsten Thue-Bludworth, 2023

![Screenshot](screenshots/iron-coder.png)

## Concept
Iron Coder is an Integrated Development Environment aimed at lowering the barrier to entry for embedded development in Rust. Inspired by modern modular hardware ecosystems, such as Adafruit's Feather specification, Sparkfun's MicroMod and QUIIC systems, and Arduino's Shield system, Iron Coder generates project templates and boilerplate code from a description of the system's hardware architecture. This gives entry-level developers a fun and easy-to-use interface to begin learning Rust, while allowing for rapid prototyping of embedded hardware/firmware systems.

This project has a few inspirations and goals:
* Engage newcomers to Rust and/or embedded development, and build a community similar to Arduino.
* Allow for community-driven additions to supported development boards.

## Project Architecture
#### `Board`s and `Project`s
A `Board`, representing a development board, is the fundamental datastructure of Iron Coder, and can be loaded into memory from the filesystem. A list of "known boards" is loaded from a specified path when Iron Coder is launched, and each Board description must contain the following items:
* A `<board-name>.toml` file, which contains information about the board.
* A `<board-name>.png` image, which will be used to display the board in the GUI.
* A `bsp` folder, which is a Rust library crate that exposes the Board's functionality to Iron Coder (TODO - document these requirements).

Rust's strong support for metaprogramming makes it a perfect language for the task of generating code. 
The following tools are used:
* `egui` for the GUI toolkit
* `syntect` for syntax highlighting
* `image` for loading images from various file types
* `serde` for loading and saving persistant state
* `toml` for loading and saving board definition files

This repository contains the code and documentation for Iron Coder.
* The [docs](./docs/) folder contains conventional documentation.
* The [mindmap](./mindmap/) folder contains Obsidian/Excalidraw brainstorming and notes.
* The [src](./src/) folder contains the Rust code for the application.
* The [assets](./assets/) folder contains fonts, icons, and other app-related assets.
* The [boards](./boards/) folder contains board definition files, images, and example projects, and is sorted by board manufacturer, then board type.

## Future Goals
* Support WASM for an online IDE, integrate Iron Coder web account for forumns and sharing ideas/code, maybe making an IoT online thing?