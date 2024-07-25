#![no_std]
#![no_main]
use rp2040_hal::pac::Peripherals;
use rp2040_hal::watchdog::Watchdog;
use rp2040_hal::sio::Sio;
use rp2040_hal::clocks::{Clock, init_clocks_and_plls};
use rp2040_hal::timer::Timer;
use rp2040_hal::gpio::{DynPinId, FunctionSioInput, Pin, Pins, PullUp};
use rp2040_hal::i2c::I2C;
use rp2040_hal::usb::UsbBus;
use rp2040_hal::fugit::{ExtU32, RateExtU32};
use usb_device::{UsbError, UsbDirection, LangID};
use usb_device::bus::{UsbBusAllocator, InterfaceNumber};
use usb_device::device::{UsbDeviceBuilder, UsbVidPid, StringDescriptors};
use usb_device::descriptor::DescriptorWriter;
use usb_device::class::UsbClass;
use usb_device::endpoint::{EndpointOut, EndpointAddress, EndpointType};
use usbd_hid::hid_class::HIDClass;
use usbd_hid::descriptor::{KeyboardReport, KeyboardUsage, SerializedDescriptor};
use embedded_hal::prelude::*;
use embedded_hal::digital::v2::InputPin;
use embedded_hal::timer::CountDown;
use ssd1306::{Ssd1306, I2CDisplayInterface};
use ssd1306::size::DisplaySize128x32;
use ssd1306::rotation::DisplayRotation;
use ssd1306::mode::DisplayConfig;
use defmt::info;
use floppapad_firmware::{VENDOR_ID, VENDOR_NAME, PRODUCT_ID, PRODUCT_NAME, BULK_OUT_ADDR};

use defmt_rtt as _;
use panic_probe as _;

#[used]
#[link_section = ".boot2"]
static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

const XOSC_FREQ: u32 = 12_000_000;

#[rp2040_hal::entry]
unsafe fn main() -> ! {
  info!("floppapad v2, firmware v{}", env!("CARGO_PKG_VERSION"));
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

  let usb_bus = UsbBusAllocator::new(UsbBus::new(
    pac.USBCTRL_REGS,
    pac.USBCTRL_DPRAM,
    clocks.usb_clock,
    true,
    &mut pac.RESETS,
  ));
  let mut silly = SillyClass::new(&usb_bus);
  let mut usb_hid = HIDClass::new(&usb_bus, KeyboardReport::desc(), 60);
  let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(VENDOR_ID, PRODUCT_ID))
    .strings(&[StringDescriptors::new(LangID::EN_US)
      .manufacturer(VENDOR_NAME)
      .product(PRODUCT_NAME)])
    .unwrap()
    .build();

  // let pins = Pins::new(
  //   pac.IO_BANK0,
  //   pac.PADS_BANK0,
  //   sio.gpio_bank0,
  //   &mut pac.RESETS,
  // );
  // let mut keys: [Pin<DynPinId, FunctionSioInput, PullUp>; 10] = [
  //   pins.gpio16.into_pull_up_input().into_dyn_pin(), // top left
  //   pins.gpio17.into_pull_up_input().into_dyn_pin(), // middle left
  //   pins.gpio18.into_pull_up_input().into_dyn_pin(), // bottom left
  //   pins.gpio19.into_pull_up_input().into_dyn_pin(), // top middle
  //   pins.gpio20.into_pull_up_input().into_dyn_pin(), // middle middle
  //   pins.gpio21.into_pull_up_input().into_dyn_pin(), // bottom middle
  //   pins.gpio24.into_pull_up_input().into_dyn_pin(), // top right
  //   pins.gpio25.into_pull_up_input().into_dyn_pin(), // middle right
  //   pins.gpio26.into_pull_up_input().into_dyn_pin(), // bottom right
  //   pins.gpio27.into_pull_up_input().into_dyn_pin(), // sigma key TODO rename
  // ];

  // let oled_i2c = I2C::i2c0(
  //   pac.I2C0,
  //   pins.gpio28.into_function(),
  //   pins.gpio29.into_function(),
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
  // oled.init().unwrap();
  // oled.print_char('f').unwrap();

  watchdog.start(1.secs());
  let mut usb_tick = timer.count_down();
  usb_tick.start(1.millis());
  let mut input_tick = timer.count_down();
  input_tick.start(100.millis());
  loop {
    if input_tick.wait().is_ok() {
      match usb_hid.push_input(&KeyboardReport {
        modifier: 0,
        reserved: 0,
        leds: 0,
        // keycodes: [KeyboardUsage::KeyboardAa as _, 0, 0, 0, 0, 0],
        keycodes: [0; 6],
      }) {
        Ok(_) | Err(UsbError::WouldBlock) => {}
        Err(e) => panic!("usb hid error: {:?}", e),
      }
    }
    if usb_tick.wait().is_ok() {
      usb_dev.poll(&mut [&mut usb_hid, &mut silly]);
    }
    watchdog.feed();
  }
}

struct SillyClass<'a> {
  interface: InterfaceNumber,
  bulk_out: EndpointOut<'a, UsbBus>,
}

impl<'a> SillyClass<'a> {
  fn new(usb_bus: &'a UsbBusAllocator<UsbBus>) -> Self {
    Self {
      interface: usb_bus.interface(),
      bulk_out: usb_bus
        .alloc(
          Some(EndpointAddress::from_parts(
            BULK_OUT_ADDR as _,
            UsbDirection::Out,
          )),
          EndpointType::Bulk,
          64,
          0,
        )
        .unwrap(),
    }
  }
}

impl UsbClass<UsbBus> for SillyClass<'_> {
  fn get_configuration_descriptors(&self, writer: &mut DescriptorWriter) -> usb_device::Result<()> {
    writer.interface(self.interface, 0xff, 0, 0)?;
    writer.endpoint(&self.bulk_out)?;
    Ok(())
  }

  fn endpoint_out(&mut self, address: EndpointAddress) {
    if self.bulk_out.address() == address {
      let mut buf = [0u8; 64];
      info!("{:?} {}", self.bulk_out.read(&mut buf), buf);
    }
  }
}
