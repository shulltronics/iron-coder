use adafruit_feather_rp2040::hal as hal;
use usb_device;
use usb_device::{
    bus::UsbBusAllocator,
    device::{UsbDevice, UsbDeviceBuilder, UsbVidPid},
};
use usbd_serial::SerialPort;

pub struct UsbManager {
    device: UsbDevice<'static, hal::usb::UsbBus>,
    serial: SerialPort<'static, hal::usb::UsbBus>,
}

impl UsbManager {
    pub fn new(usb_bus: &'static UsbBusAllocator<hal::usb::UsbBus>) -> Self {
        
        let serial = usbd_serial::SerialPort::new(usb_bus);

        let device = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x2E8A, 0x000a))
            .manufacturer("Raspberry Pi")
            .product("Pico")
            .serial_number("TEST")
            .device_class(2)
			.max_packet_size_0(64)	// yay! works on windows :)
            .build();

        UsbManager { device, serial }
    }

    pub unsafe fn interrupt(&mut self) {
        rtt_target::rprintln!("in interrupt");
        if self.device.poll(&mut [&mut self.serial]) {
            let mut buf = [0u8; 64];
            match self.serial.read(&mut buf) {
                Err(_e) => {
                    // Do nothing
                }
                Ok(_count) => {
                    // Do nothing
                }
            }
        }
    }
}

impl core::fmt::Write for UsbManager {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        critical_section::with(|_| {
            // Now interrupts are disabled
            match self.serial.write(s.as_bytes()) {
                Err(_e) => {
                    // Do nothing
                }
                Ok(_) => {
                    // Do nothing
                }
            }
        });
        Ok(())
    }
}