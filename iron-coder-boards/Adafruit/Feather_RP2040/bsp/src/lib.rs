#![no_std]

pub mod prelude;
use embedded_hal::blocking::i2c;
use prelude::*;

use adafruit_feather_rp2040 as feather;

use feather::{Pins, XOSC_CRYSTAL_FREQ};
use feather::hal::{
	pac,
	pac::interrupt,
    clocks::{init_clocks_and_plls, Clock},
    watchdog::Watchdog,
	timer::Timer,
    Sio,
	gpio::pin::Pin,
	gpio::pin::PushPullOutput,
	gpio::pin::bank0::*,		// all gpios into scope
	gpio::FunctionI2C,
	gpio::FunctionSpi,
	I2C,
	Spi,
	pio::PIOExt,
};

use fugit::RateExtU32;

// a SYSTIC-based delay timer
use cortex_m::delay::Delay;
// struct for neopixels
use ws2812_pio::Ws2812Direct;

// USB Device support
use usb_device::class_prelude::*;
// USB Communications Class Device support
mod usb_manager;
use usb_manager::UsbManager;
use core::fmt::Write as _;

// Global USB objects & interrupt
static mut USB_BUS: Option<UsbBusAllocator<feather::hal::usb::UsbBus>> = None;
static mut USB_MANAGER: Option<UsbManager> = None;
#[allow(non_snake_case)]
#[interrupt]
unsafe fn USBCTRL_IRQ() {
    match USB_MANAGER.as_mut() {
        Some(manager) => manager.interrupt(),
        None => (),
    };
}

pub type OnboardLed = Pin<Gpio13, PushPullOutput>;

type OnboardNeopixel = Ws2812Direct<
    feather::hal::pac::PIO0,
    feather::hal::pio::SM0,
	feather::hal::gpio::pin::bank0::Gpio16,
>;

pub type I2CBus = I2C<
	pac::I2C1,
	(Pin<Gpio2, FunctionI2C>, Pin<Gpio3, FunctionI2C>)
>;

pub type SPIBus = Spi<
	adafruit_feather_rp2040::hal::spi::Enabled,
	pac::SPI0,
	8,
>;

pub type Cs = Pin<Gpio6, PushPullOutput>;
pub type Rst = Pin<Gpio7, PushPullOutput>;
pub type Dc = Pin<Gpio25, PushPullOutput>;

pub struct Board {
	// pub pins: Pins,
	pub d4: Option<Pin<Gpio6, PushPullOutput>>,
	pub d5: Option<Pin<Gpio7, PushPullOutput>>,
	pub d24: Option<Pin<Gpio24, PushPullOutput>>,
	pub d25: Option<Pin<Gpio25, PushPullOutput>>,
	pub red_led: OnboardLed,
	neopixel: OnboardNeopixel,
	pub i2c_manager: shared_bus::BusManagerSimple<I2CBus>,//Option<I2CBus>,
	pub spi_bus: Option<SPIBus>,
	pub delay_timer: Delay,
	pub test: Option<bool>,
	usb: &'static mut UsbManager,
}

impl Board {
	pub fn new() -> Self {
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
    	).ok().unwrap();

   	 // Setup USB
    	let usb = unsafe {
        	USB_BUS = Some(UsbBusAllocator::new(feather::hal::usb::UsbBus::new(
            	pac.USBCTRL_REGS,
            	pac.USBCTRL_DPRAM,
            	clocks.usb_clock,
            	true,
            	&mut pac.RESETS,
        	)));
        	USB_MANAGER = Some(UsbManager::new(USB_BUS.as_ref().unwrap()));
        	// Enable the USB interrupt
        	feather::pac::NVIC::unmask(feather::hal::pac::Interrupt::USBCTRL_IRQ);
        	USB_MANAGER.as_mut().unwrap()
    	};

		// initialize the Single Cycle IO
    	let sio = Sio::new(pac.SIO);
    	// initialize the pins to default state
    	let pins = Pins::new(
        	pac.IO_BANK0,
        	pac.PADS_BANK0,
        	sio.gpio_bank0,
        	&mut pac.RESETS,
    	);

		// Setup the I2C1 instance, connected to the SCL/SDA pins on the Feather
		let scl = pins.scl.into_mode::<FunctionI2C>();
		let sda = pins.sda.into_mode::<FunctionI2C>();
		let i2c1: I2CBus = I2C::i2c1(
			pac.I2C1,
			sda,
			scl,
			RateExtU32::kHz(400),
			&mut pac.RESETS,
			&clocks.system_clock,
		);
		// Create the bus manager
		let i2c_manager = shared_bus::BusManagerSimple::new(i2c1);

		let mosi = pins.mosi.into_mode::<FunctionSpi>();
		let miso = pins.miso.into_mode::<FunctionSpi>();
		let sclk = pins.sclk.into_mode::<FunctionSpi>();
		let spi = Spi::<_, _, 8>::new(pac.SPI0);

		// Exchange the uninitialised SPI driver for an initialised one
		let mut spi = spi.init(
			&mut pac.RESETS,
			clocks.peripheral_clock.freq(),
			16.MHz(),
			&embedded_hal::spi::MODE_0,
		);

		// setup the general-purpose delay timer
		let dt = Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

		// setup the on-board neopixel
		let _timer = Timer::new(pac.TIMER, &mut pac.RESETS);
		let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
		let np = Ws2812Direct::new(
        	pins.neopixel.into_mode(),
        	&mut pio,
        	sm0,
        	clocks.peripheral_clock.freq(),
        	//timer.count_down(),
    	);

		Self {
			d4: Some(pins.d4.into_mode::<PushPullOutput>()),
			d5: Some(pins.d5.into_mode::<PushPullOutput>()),
			d24: Some(pins.d24.into_mode::<PushPullOutput>()),
			d25: Some(pins.d25.into_mode::<PushPullOutput>()),
			red_led: pins.d13.into_push_pull_output(),
			neopixel: np,
			i2c_manager: i2c_manager,//Some(i2c1),
			spi_bus: Some(spi),
			delay_timer: dt,
			test: Some(true),
			usb: usb,
		}
	}

	pub fn delay(&mut self, ms: u32) {
		self.delay_timer.delay_ms(ms);
	}

	pub fn set_led(&mut self, state: bool) {
		match state {
			true  => self.red_led.set_high().unwrap(),
			false => self.red_led.set_low().unwrap(),
		}
	}

	pub fn set_neopixel_color(&mut self, color: smart_leds::RGB8) {
		let _ = self.neopixel.write(brightness([color].iter().cloned(), 50));
	}

	pub fn serial_write(&mut self, s: impl core::fmt::Debug) {
		write!(self.usb, "{:?}\r\n", s).unwrap();
	}

}