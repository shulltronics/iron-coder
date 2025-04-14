//! This assumes that a LED is connected to the pin assigned to `led`. (GPIO4)
//TODO: explore how to pull pinout from the board being used or decide the example board

use esp_backtrace as _;
use esp_println::println;
use esp_hal::{
    delay::Delay,
    gpio::{Io, Level, Output},
    peripherals::Peripherals,
    prelude::*,
    system::SystemControl,
};

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::max(system.clock_control).freeze();

    // Set the LED as an output and set it to high initially (active low).
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut led = Output::new(io.pins.gpio4, Level::High);

    led.set_high();

    // Initialize Delay peripheral, use it to toggle the LED state within loop.
    let mut delay = Delay::new(&clocks);
    loop {
        delay.delay_millis(1000);
        led.toggle();
    }
}