#![no_std]
#![no_main]

/***** low-level imports *****/
use panic_halt as _;
use embedded_hal::{
    digital::{OutputPin, InputPin},
    spi,
    delay::DelayNs,
};

/***** board-specific imports *****/
use adafruit_feather_rp2040::entry;
use adafruit_feather_rp2040::{
    hal::{
        clocks::{init_clocks_and_plls, Clock},
        pac,
        watchdog::Watchdog,
        Sio,
        gpio::{FunctionUart, FunctionSpi, FunctionI2C},
        uart,
        I2C,
        pio::PIOExt,
        timer::Timer,
    },
    Pins, XOSC_CRYSTAL_FREQ,
};

/***** imports for external devices *****/
use ws2812_pio::Ws2812;
use smart_leds::{brightness, SmartLedsWrite, RGB8};

#[entry]
fn main() -> ! {
    // Grab the singleton objects
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    // Init the watchdog timer, to pass into the clock init 
    let mut watchdog = Watchdog::new(pac.WATCHDOG);

    let clocks = init_clocks_and_plls(
        XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    ).ok().unwrap();

    // initialize the Single Cycle IO
    let sio = Sio::new(pac.SIO);
    // initialize the pins to default state 
    let pins = Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    // Setup the Propmaker Power Enable Pin (pin 10)
    let mut pwr_pin = pins.d10.into_push_pull_output();
    pwr_pin.set_high().unwrap();

    // Configure the addressable LED
    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
    let mut neopixels = Ws2812::new(
        pins.d5.into_function(),
        &mut pio,
        sm0,
        clocks.peripheral_clock.freq(),
        timer.count_down(),
    );

    // Infinite colour wheel loop
    let mut n: u8 = 128;
    let mut timer = timer; // rebind to force a copy of the timer
    loop {
        // Create a fixed-size array for the 64 LEDs
        let mut colors: [RGB8; 64] = [RGB8::default(); 64];
        for (i, color) in colors.iter_mut().enumerate() {
            // Use `255` as the modulo to avoid the out-of-range error
            *color = wheel((n + i as u8) % 255);
        }

        // Write to the NeoPixel LEDs
        neopixels.write(brightness(colors.iter().cloned(), 32)).unwrap();
        
        n = n.wrapping_add(1);

        // Delay between updates
        timer.delay_ms(25);
    }
}

/// Convert a number from `0..=255` to an RGB color triplet.
/// The colours are a transition from red, to green, to blue and back to red.
fn wheel(mut wheel_pos: u8) -> RGB8 {
    wheel_pos = 255 - wheel_pos;
    if wheel_pos < 85 {
        // No green in this sector - red and blue only
        (255 - (wheel_pos * 3), 0, wheel_pos * 3).into()
    } else if wheel_pos < 170 {
        // No red in this sector - green and blue only
        wheel_pos -= 85;
        (0, wheel_pos * 3, 255 - (wheel_pos * 3)).into()
    } else {
        // No blue in this sector - red and green only
        wheel_pos -= 170;
        (wheel_pos * 3, 255 - (wheel_pos * 3), 0).into()
    }
}
