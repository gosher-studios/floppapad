#![no_std]
#![no_main]
use rp2040_hal::clocks::{Clock, init_clocks_and_plls};
use rp2040_hal::pac::Peripherals;
use rp2040_hal::sio::Sio;
use rp2040_hal::watchdog::Watchdog;
use rp2040_hal::gpio::Pins;
// use rp2040_hal::i2c::I2C;
// use rp2040_hal::usb::UsbBus;
use rp2040_hal::fugit::{ExtU32, RateExtU32};
// use usb_device::bus::UsbBusAllocator;
// use usb_device::device::{UsbDeviceBuilder, UsbVidPid};
use embedded_hal::prelude::*;
// use ssd1306::{Ssd1306, I2CDisplayInterface};
// use ssd1306::size::DisplaySize128x32;
// use ssd1306::rotation::DisplayRotation;
// use ssd1306::mode::DisplayConfig;

use defmt_rtt as _;
use panic_probe as _;

#[used]
#[link_section = ".boot2"]
static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

const XOSC_FREQ: u32 = 12_000_000;
// const PRODUCT_NAME: &str = "floppapad";
// const PRODUCT_ID: u16 = 0xf19d;
// const VENDOR_NAME: &str = "gosher studios";
// const VENDOR_ID: u16 = 0xf109;

#[rp2040_hal::entry]
unsafe fn main() -> ! {
  let mut pac = Peripherals::take().unwrap();
  let mut watchdog = Watchdog::new(pac.WATCHDOG);
  let sio = Sio::new(pac.SIO);
  let clocks = init_clocks_and_plls(
    XOSC_FREQ,
    pac.XOSC,
    pac.CLOCKS,
    pac.PLL_SYS,
    pac.PLL_USB,
    &mut pac.RESETS,
    &mut watchdog,
  )
  .ok()
  .unwrap();
  let pins = Pins::new(
    pac.IO_BANK0,
    pac.PADS_BANK0,
    sio.gpio_bank0,
    &mut pac.RESETS,
  );

  // let usb_bus = UsbBusAllocator::new(UsbBus::new(
  //   pac.USBCTRL_REGS,
  //   pac.USBCTRL_DPRAM,
  //   clocks.usb_clock,
  //   true,
  //   &mut pac.RESETS,
  // ));
  // let mut usb_serial = SerialPort::new(USB_BUS.assume_init_ref());
  // let mut usb_dev =
  //   UsbDeviceBuilder::new(USB_BUS.assume_init_ref(), UsbVidPid(VENDOR_ID, PRODUCT_ID))
  //     .product(PRODUCT_NAME)
  //     .manufacturer(VENDOR_NAME)
  //     .device_class(2)
  //     .build();

  // let oled_i2c = I2C::i2c0(
  //   pac.I2C0,
  //   // pins.gpio28.into_function(),
  //   // pins.gpio29.into_function(),
  //   pins.gpio0.into_function(),
  //   pins.gpio1.into_function(),
  //   400.kHz(),
  //   &mut pac.RESETS,
  //   clocks.system_clock.freq(),
  // );
  // let mut oled = Ssd1306::new(
  //   I2CDisplayInterface::new(oled_i2c),
  //   DisplaySize128x32,
  //   DisplayRotation::Rotate0,
  // )
  // .into_terminal_mode();
  //    oled.init().unwrap();
  //    oled.print_char('a').unwrap();

  defmt::info!("hello floppa");
  watchdog.start(1.secs());
  loop {
    // if usb_dev.poll(&mut []) {
    // }
    watchdog.feed();
  }
}
