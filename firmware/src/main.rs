#![no_std]
#![no_main]
use core::panic::PanicInfo;
use rp2040_hal::clocks::{Clock, init_clocks_and_plls};
use rp2040_hal::pac::Peripherals;
use rp2040_hal::sio::Sio;
use rp2040_hal::watchdog::Watchdog;
use rp2040_hal::gpio::Pins;
use rp2040_hal::i2c::I2C;
use rp2040_hal::usb::UsbBus;
use rp2040_hal::fugit::RateExtU32;
use usb_device::bus::UsbBusAllocator;
use usb_device::device::{UsbDeviceBuilder, UsbVidPid};
use usbd_serial::SerialPort;
use ssd1306::{Ssd1306, I2CDisplayInterface};
use ssd1306::size::DisplaySize128x32;
use ssd1306::rotation::DisplayRotation;
use ssd1306::mode::DisplayConfig;

#[used]
#[link_section = ".boot2"]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

const XOSC_FREQ: u32 = 12_000_000;
const PRODUCT_NAME: &str = "floppapad";
const PRODUCT_ID: u16 = 0xf19d;
const VENDOR_NAME: &str = "gosher studios";
const VENDOR_ID: u16 = 0xf109;

#[rp2040_hal::entry]
fn main() -> ! {
  let mut peripherals = Peripherals::take().unwrap();
  let mut watchdog = Watchdog::new(peripherals.WATCHDOG);
  let sio = Sio::new(peripherals.SIO);
  let clocks = init_clocks_and_plls(
    XOSC_FREQ,
    peripherals.XOSC,
    peripherals.CLOCKS,
    peripherals.PLL_SYS,
    peripherals.PLL_USB,
    &mut peripherals.RESETS,
    &mut watchdog,
  )
  .ok()
  .unwrap();
  let pins = Pins::new(
    peripherals.IO_BANK0,
    peripherals.PADS_BANK0,
    sio.gpio_bank0,
    &mut peripherals.RESETS,
  );

  let usb_bus = UsbBusAllocator::new(UsbBus::new(
    peripherals.USBCTRL_REGS,
    peripherals.USBCTRL_DPRAM,
    clocks.usb_clock,
    true,
    &mut peripherals.RESETS,
  ));
  let usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(VENDOR_ID, PRODUCT_ID))
    .product(PRODUCT_NAME)
    .manufacturer(VENDOR_NAME)
    .device_class(2)
    .build();
  let mut usb_serial = SerialPort::new(&usb_bus);
  usb_serial.write("hello floppa".as_bytes()).unwrap();

  let oled_i2c = I2C::i2c0(
    peripherals.I2C0,
    pins.gpio28.into_function(),
    pins.gpio29.into_function(),
    400.kHz(),
    &mut peripherals.RESETS,
    clocks.system_clock.freq(),
  );
  let mut oled = Ssd1306::new(
    I2CDisplayInterface::new(oled_i2c),
    DisplaySize128x32,
    DisplayRotation::Rotate0,
  )
  .into_terminal_mode();
  oled.init().unwrap();
  oled.print_char('a').unwrap();

  rp2040_hal::halt();
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
  rp2040_hal::halt();
}
