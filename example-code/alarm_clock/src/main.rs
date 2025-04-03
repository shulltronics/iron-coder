#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

/*
 This is derived from Rahix' comment to
https://github.com/Rahix/avr-hal/issues/75
and then modernized to account for API drift since 2020

*/

use arduino_hal::port::mode::{Output, PwmOutput};
use arduino_hal::port::Pin;
use arduino_hal::prelude::*;
use arduino_hal::simple_pwm::*;
use avr_device::atmega328p::tc1::tccr1b::CS1_A;
use avr_device::atmega328p::TC1;
use core::mem;
use core::sync::atomic::{AtomicBool, Ordering};
use core::sync::atomic::Ordering::SeqCst;
use arduino_hal::delay_ms;
use panic_halt as _;
use ufmt::{uWrite, uwriteln};

struct InterruptState {
    blinker: Pin<Output>,
}

static ALARM_ON: AtomicBool = AtomicBool::new(false);
static PIN_CHANGED: AtomicBool = AtomicBool::new(false);

static mut INTERRUPT_STATE: mem::MaybeUninit<InterruptState> = mem::MaybeUninit::uninit();

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let timer0 = Timer0Pwm::new(dp.TC0, Prescaler::Prescale64);
    let mut pwm_led = pins.d5.into_output().into_pwm(&timer0);

    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    ufmt::uwriteln!(&mut serial, "Hello from Arduino!\r").unwrap_infallible();

    let mut button_pin = pins.d2.into_pull_up_input();
    let mut led = pins.d13.into_output();

    // Enable the PCINT2 pin change interrupt
    dp.EXINT.pcicr.write(|w| unsafe { w.bits(0b100) });

    // Enable pin change interrupts on PCINT18 which is pin PD2 (= d2)
    dp.EXINT.pcmsk2.write(|w| w.bits(0b100));


    let tmr1: TC1 = dp.TC1;

    init_timer(&tmr1, &mut serial);

    // Enable interrupts globally, not a replacement for the specific interrupt enable
    unsafe {
        // SAFETY: Not inside a critical section and any non-atomic operations have been completed
        // at this point.
        avr_device::interrupt::enable();
    }

    ufmt::uwriteln!(
        &mut serial,
        "configured timer output compare register = {}",
        tmr1.ocr1a.read().bits()
    )
        .unwrap_infallible();

    loop {
        if(rotate(&PIN_CHANGED)){
            ALARM_ON.store(false, SeqCst);
            tmr1.tcnt1.reset();
            tmr1.timsk1.write(|w| w.ocie1a().set_bit());
            led.toggle();
            PIN_CHANGED.store(false, SeqCst);
        }
        if(rotate(&ALARM_ON)){
            tmr1.timsk1.write(|w| w.ocie1a().clear_bit());
            pwm_led.enable();
            pwm_led.set_duty(150);
            pwm_led.disable();
        }
    }
}

pub fn init_timer<W: uWrite<Error = ::core::convert::Infallible>>(tmr1: &TC1, serial: &mut W) {
    /*
     https://ww1.microchip.com/downloads/en/DeviceDoc/Atmel-7810-Automotive-Microcontrollers-ATmega328P_Datasheet.pdf
     section 15.11
    */
    use arduino_hal::clock::Clock;

    const ARDUINO_UNO_CLOCK_FREQUENCY_HZ: u32 = arduino_hal::DefaultClock::FREQ;
    const CLOCK_SOURCE: CS1_A = CS1_A::PRESCALE_1024;
    let clock_divisor: u32 = match CLOCK_SOURCE {
        CS1_A::DIRECT => 1,
        CS1_A::PRESCALE_8 => 8,
        CS1_A::PRESCALE_64 => 64,
        CS1_A::PRESCALE_256 => 256,
        CS1_A::PRESCALE_1024 => 1024,
        CS1_A::NO_CLOCK | CS1_A::EXT_FALLING | CS1_A::EXT_RISING => {
            uwriteln!(serial, "uhoh, code tried to set the clock source to something other than a static prescaler {}", CLOCK_SOURCE as usize)
                .unwrap_infallible();
            1
        }
    };

    let ticks :u16 = 65535;
    ufmt::uwriteln!(
        serial,
        "configuring timer output compare register = {}",
        ticks
    )
        .unwrap_infallible();

    // Timer Control Registers
    tmr1.tccr1a.write(|w| w.wgm1().bits(0b00));
    tmr1.tccr1b.write(|w| {
        w.cs1()
            //.prescale_256()
            .variant(CLOCK_SOURCE)
            .wgm1()
            .bits(0b01)
    });
    // Write "ticks" to Compare Register A
    tmr1.ocr1a.write(|w| w.bits(ticks));
    // Enable Compare Match A Interrupts
    tmr1.timsk1.write(|w| w.ocie1a().set_bit());
}

fn rotate(flag: &AtomicBool) -> bool {
    avr_device::interrupt::free(|_cs| {
        if flag.load(Ordering::SeqCst) {
            true
        } else {
            false
        }
    })
}

#[avr_device::interrupt(atmega328p)]
fn TIMER1_COMPA() {
    ALARM_ON.store(true, Ordering::SeqCst);
}

//This function is called on change of pin 2
#[avr_device::interrupt(atmega328p)]
#[allow(non_snake_case)]
fn PCINT2() {
    PIN_CHANGED.store(true, Ordering::SeqCst);
}