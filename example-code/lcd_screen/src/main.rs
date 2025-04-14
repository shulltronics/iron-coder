//! Blinks the LED on a Adafruit Feather RP2040 board
//!
//! This will blink on-board LED.
#![no_std]				
#![no_main]	

use adafruit_feather_rp2040::entry;
use adafruit_feather_rp2040::{
    hal::{
        clocks::{init_clocks_and_plls, Clock},
        pac,
        watchdog::Watchdog,
        Sio,
		gpio::Pins,
		i2c::I2C,	
    },
    XOSC_CRYSTAL_FREQ,
};
use embedded_hal::digital::v2::OutputPin;
use panic_halt as _;
use fugit::RateExtU32;

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    let mut watchdog = Watchdog::new(pac.WATCHDOG);

    let clocks = init_clocks_and_plls(
        XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let sio = Sio::new(pac.SIO);
    let pins = Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
		

	// Set i2c peripherial for communiction
	let i2c_periph = pac.I2C1;
	// Set sda and scl lines using gpio pins (reconfigure them to I2C functions)
	let sda = pins.gpio18.into_function::<adafruit_feather_rp2040::hal::gpio::FunctionI2c>();	
	let scl = pins.gpio19.into_function::<adafruit_feather_rp2040::hal::gpio::FunctionI2c>();
		
	// Create i2c module for reading and writing
	let mut i2c = I2C::i2c1(
    	i2c_periph,
    	sda, // sda
    	scl, // scl
    	400.kHz(),
    	&mut pac.RESETS,
    	125_000_000.Hz(),			
	);			
		
	let addr = 0x3Cu8;
	
	// Turn LCD screen on
	 
		
    loop {

    }
}