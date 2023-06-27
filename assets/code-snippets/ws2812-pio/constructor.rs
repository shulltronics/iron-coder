let mut neopixels = Ws2812::new(
    pins.d5.into_mode(),
    &mut pio,
    sm0,
    clocks.peripheral_clock.freq(),
    timer.count_down(),
);