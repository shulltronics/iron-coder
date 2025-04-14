#![no_std]
#![no_main]

/**** low-level imports *****/
use core::fmt::Write;
use core::panic::PanicInfo;
// use panic_halt as _;
// use cortex_m::prelude::*;
use cortex_m_rt::entry;
use embedded_hal::{
    digital::v2::{OutputPin},
};

/***** board-specific imports *****/
use adafruit_feather_rp2040::hal as hal;
use hal::{
    pac::interrupt,
    clocks::{init_clocks_and_plls, Clock},
    pac,
    watchdog::Watchdog,
    Sio,
};
use adafruit_feather_rp2040::{
    Pins, XOSC_CRYSTAL_FREQ,
};

// USB Device support
use usb_device::class_prelude::*;
// USB Communications Class Device support
mod usb_manager;
use usb_manager::UsbManager;
// Global USB objects & interrupt
static mut USB_BUS: Option<UsbBusAllocator<hal::usb::UsbBus>> = None;
static mut USB_MANAGER: Option<UsbManager> = None;
#[allow(non_snake_case)]
#[interrupt]
unsafe fn USBCTRL_IRQ() {
    match USB_MANAGER.as_mut() {
        Some(manager) => manager.interrupt(),
        None => (),
    };
}
#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    if let Some(usb) = unsafe { USB_MANAGER.as_mut() } {
        writeln!(usb, "{}", panic_info).ok();
    }
    loop {}
}

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
    
    // Setup USB
    let usb = unsafe {
        USB_BUS = Some(UsbBusAllocator::new(hal::usb::UsbBus::new(
            pac.USBCTRL_REGS,
            pac.USBCTRL_DPRAM,
            clocks.usb_clock,
            true,
            &mut pac.RESETS,
        )));
        USB_MANAGER = Some(UsbManager::new(USB_BUS.as_ref().unwrap()));
        // Enable the USB interrupt
        pac::NVIC::unmask(hal::pac::Interrupt::USBCTRL_IRQ);
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

    let mut timer = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
    let mut led_pin = pins.d13.into_push_pull_output();

    /*
    Loop Section
    */
    let delay: u32 = 500;   // loop delay in ms
    let mut n: u32 = 0;
    loop {
        write!(usb, "starting loop number {:?}\r\n", n).unwrap();
        led_pin.set_low().unwrap();
        timer.delay_ms(delay as u32);
        led_pin.set_high().unwrap();
        timer.delay_ms(delay as u32);
        n = n + 1;
    }

}