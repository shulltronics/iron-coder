#![no_std]
#![no_main]

mod system;
use system::System;

#[entry]
fn main() -> ! {
    let mut system = System::new();
    // put your setup code here!

    loop {
        // put your look code here!
    }
}