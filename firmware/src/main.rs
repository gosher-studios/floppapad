#![no_std]
#![no_main]
use core::fmt::Write;

use rp2040_hal::pac::Peripherals;
use rp2040_hal::watchdog::Watchdog;
use rp2040_hal::sio::Sio;
use rp2040_hal::clocks::{Clock, init_clocks_and_plls};
use rp2040_hal::timer::Timer;
use rp2040_hal::gpio::{DynPinId, FunctionSioInput, Pin, Pins, PullUp};
use rp2040_hal::i2c::I2C;
use rp2040_hal::usb::UsbBus;
use rp2040_hal::fugit::{ExtU32, RateExtU32};
use usb_device::bus::UsbBusAllocator;
use usb_device::device::{UsbDeviceBuilder, UsbVidPid};
use usbd_human_interface_device::usb_class::UsbHidClassBuilder;
use usbd_human_interface_device::device::keyboard::NKROBootKeyboardConfig;
use usbd_human_interface_device::page::Keyboard;
use embedded_hal::prelude::*;
use embedded_hal::digital::v2::InputPin;
use ssd1306::{Ssd1306, I2CDisplayInterface};
use ssd1306::size::DisplaySize128x32;
use ssd1306::rotation::DisplayRotation;
use ssd1306::mode::DisplayConfig;
use floppapad_firmware::{PRODUCT_ID, PRODUCT_NAME, VENDOR_ID, VENDOR_NAME};

use defmt_rtt as _;
use panic_probe as _;
use usbd_human_interface_device::UsbHidError;

#[used]
#[link_section = ".boot2"]
static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

const XOSC_FREQ: u32 = 12_000_000;

#[rp2040_hal::entry]
unsafe fn main() -> ! {
  defmt::info!("hello floppa");
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
  let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
  let pins = Pins::new(
    pac.IO_BANK0,
    pac.PADS_BANK0,
    sio.gpio_bank0,
    &mut pac.RESETS,
  );

  let usb_alloc = UsbBusAllocator::new(UsbBus::new(
    pac.USBCTRL_REGS,
    pac.USBCTRL_DPRAM,
    clocks.usb_clock,
    true,
    &mut pac.RESETS,
  ));
  let mut usb_hid = UsbHidClassBuilder::new()
    .add_device(NKROBootKeyboardConfig::default())
    .build(&usb_alloc);
  let mut usb_dev = UsbDeviceBuilder::new(&usb_alloc, UsbVidPid(VENDOR_ID, PRODUCT_ID))
    .product(PRODUCT_NAME)
    .manufacturer(VENDOR_NAME)
    .serial_number("TEST")
    .build();

  let mut keys: [Pin<DynPinId, FunctionSioInput, PullUp>; 10] = [
    pins.gpio16.into_pull_up_input().into_dyn_pin(), // top left
    pins.gpio17.into_pull_up_input().into_dyn_pin(), // middle left
    pins.gpio18.into_pull_up_input().into_dyn_pin(), // bottom left
    pins.gpio19.into_pull_up_input().into_dyn_pin(), // top middle
    pins.gpio20.into_pull_up_input().into_dyn_pin(), // middle middle
    pins.gpio21.into_pull_up_input().into_dyn_pin(), // bottom middle
    pins.gpio24.into_pull_up_input().into_dyn_pin(), // top right
    pins.gpio25.into_pull_up_input().into_dyn_pin(), // middle right
    pins.gpio26.into_pull_up_input().into_dyn_pin(), // bottom right
    pins.gpio27.into_pull_up_input().into_dyn_pin(), // sigma key TODO rename
  ];
  let oled_i2c = I2C::i2c0(
    pac.I2C0,
    pins.gpio28.into_function(),
    pins.gpio29.into_function(),
    400.kHz(),
    &mut pac.RESETS,
    clocks.system_clock.freq(),
  );
  let mut oled = Ssd1306::new(
    I2CDisplayInterface::new(oled_i2c),
    DisplaySize128x32,
    DisplayRotation::Rotate0,
  )
  .into_terminal_mode();
  oled.init().unwrap();
  oled.print_char('f').unwrap();

  watchdog.start(1.secs());
  let mut usb_tick = timer.count_down();
  usb_tick.start(1.millis());
  let mut input_tick = timer.count_down();
  input_tick.start(10.millis());
  loop {
    if input_tick.wait().is_ok() {
      let keys = get_keys(&mut keys);
      for i in 0..9 {
        if !(keys[i] == Keyboard::NoEventIndicated) {
          oled.write_char('e').unwrap();
          oled.clear().unwrap();
        }
      }
      match usb_hid.device().write_report(keys) {
        Ok(_) => {}
        Err(UsbHidError::WouldBlock) => {}
        Err(UsbHidError::Duplicate) => {}
        Err(e) => core::panic!("joever {:?}", e),
      };
    }
    if usb_tick.wait().is_ok() {
      match usb_hid.tick() {
        Ok(_) => {}
        Err(UsbHidError::WouldBlock) => {}
        Err(e) => core::panic!("joever {:?}", e),
      };
    }
    if usb_dev.poll(&mut [&mut usb_hid]) {
      match usb_hid.device().read_report() {
        Ok(_) => {}
        Err(_) => {}
      };
    }
    watchdog.feed();
  }
}

fn get_keys(keys: &mut [Pin<DynPinId, FunctionSioInput, PullUp>]) -> [Keyboard; 10] {
  let mut key_codes = [
    Keyboard::A,
    Keyboard::B,
    Keyboard::C,
    Keyboard::D,
    Keyboard::E,
    Keyboard::F,
    Keyboard::G,
    Keyboard::H,
    Keyboard::I,
    Keyboard::J,
  ];
  for i in 0..9 {
    if !keys[i].is_low().unwrap() {
      key_codes[i] = Keyboard::NoEventIndicated
    }
  }
  key_codes
}
