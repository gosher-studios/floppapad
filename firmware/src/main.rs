#![no_std]
#![no_main]
extern crate alloc;

use core::str;
use core::fmt::Write;
use core::mem::MaybeUninit;
use alloc::string::String;
use alloc::vec::Vec;
use rp2040_hal::pac::Peripherals;
use rp2040_hal::watchdog::Watchdog;
use rp2040_hal::sio::Sio;
use rp2040_hal::clocks::{Clock, init_clocks_and_plls};
use rp2040_hal::timer::Timer;
use rp2040_hal::gpio::{DynPinId, FunctionSioInput, Pin, Pins, PullUp};
use rp2040_hal::i2c::I2C;
use rp2040_hal::usb::UsbBus;
use rp2040_hal::fugit::{ExtU32, RateExtU32};
use rp2040_flash::flash::flash_unique_id;
use cortex_m::interrupt;
use embedded_hal::prelude::*;
use embedded_hal::digital::v2::InputPin;
use embedded_hal::timer::CountDown;
use embedded_alloc::Heap;
use usb_device::{UsbError, UsbDirection, LangID};
use usb_device::bus::{UsbBusAllocator, InterfaceNumber};
use usb_device::device::{UsbDeviceBuilder, UsbVidPid, StringDescriptors};
use usb_device::descriptor::DescriptorWriter;
use usb_device::class::{UsbClass, ControlOut};
use usb_device::endpoint::{EndpointOut, EndpointAddress, EndpointType};
use usb_device::control::RequestType;
use usbd_hid::hid_class::HIDClass;
use usbd_hid::descriptor::{KeyboardReport, KeyboardUsage, SerializedDescriptor};
use ssd1306::{Ssd1306, I2CDisplayInterface};
use ssd1306::size::DisplaySize128x32;
use ssd1306::rotation::DisplayRotation;
use ssd1306::mode::DisplayConfig;
use rhai::{Engine, AST, Scope};
use rhai::packages::{Package, CorePackage};
use defmt::{info, debug, warn};
use floppapad_firmware::{ControlType, VENDOR_ID, VENDOR_NAME, PRODUCT_ID, PRODUCT_NAME, BULK_OUT_ADDR};

use defmt_rtt as _;
use panic_probe as _;

const XOSC_FREQ: u32 = 12_000_000;
const HEAP_SIZE: usize = 1024 * 128;

#[used]
#[link_section = ".boot2"]
static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;
#[global_allocator]
static HEAP: Heap = Heap::empty();
static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];

#[rp2040_hal::entry]
unsafe fn main() -> ! {
  info!("floppapad v2, firmware v{}", env!("CARGO_PKG_VERSION"));
  HEAP.init(HEAP_MEM.as_ptr() as _, HEAP_SIZE);
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
  let mut usb_control = ControlClass::new(&usb_bus);
  let mut usb_hid = HIDClass::new(&usb_bus, KeyboardReport::desc(), 60);
  let serial_number = get_serial_number();
  let version_bcd = get_version_bcd();
  let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(VENDOR_ID, PRODUCT_ID))
    .strings(&[StringDescriptors::new(LangID::EN_US)
      .manufacturer(VENDOR_NAME)
      .product(PRODUCT_NAME)
      .serial_number(&serial_number)])
    .unwrap()
    .device_release(version_bcd)
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

  let mut engine = Engine::new_raw();
  engine.set_max_strings_interned(1024);
  engine.on_print(|s| info!("rhai: {}", s));
  engine.on_debug(|s, _, _| debug!("rhai: {}", s));
  CorePackage::new().register_into_engine(&mut engine);
  let mut ast = AST::empty();
  let mut scope = Scope::new();

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
      if usb_dev.poll(&mut [&mut usb_control, &mut usb_hid]) {
        if let Some(src) = usb_control.buf.take_if(|b| b.len() == b.capacity()) {
          match engine.compile(str::from_utf8(&src).unwrap()) {
            Ok(a) => {
              ast = a;
              engine.call_fn::<()>(&mut scope, &ast, "init", ()).unwrap();
            }
            Err(e) => warn!("compile error {}", defmt::Display2Format(&e)),
          }
        }
      }
    }
    watchdog.feed();
  }
}

struct ControlClass<'a> {
  interface: InterfaceNumber,
  bulk_out: EndpointOut<'a, UsbBus>,
  buf: Option<Vec<u8>>,
}

impl<'a> ControlClass<'a> {
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
      buf: None,
    }
  }
}

impl UsbClass<UsbBus> for ControlClass<'_> {
  fn get_configuration_descriptors(&self, writer: &mut DescriptorWriter) -> usb_device::Result<()> {
    writer.interface(self.interface, 0xff, 0, 0)?;
    writer.endpoint(&self.bulk_out)?;
    Ok(())
  }

  fn control_out(&mut self, xfer: ControlOut<UsbBus>) {
    if xfer.request().request_type == RequestType::Vendor {
      match ControlType::from(xfer.request().request) {
        ControlType::PreScript => {
          let len = xfer.request().value;
          self.buf = Some(Vec::with_capacity(len as _));
          debug!("allocated {} for script", len);
          xfer.accept().unwrap();
        }
        ControlType::Test => {
          info!("test");
          xfer.accept().unwrap()
        }
        _ => xfer.reject().unwrap(),
      }
    }
  }

  fn endpoint_out(&mut self, address: EndpointAddress) {
    if self.bulk_out.address() == address {
      if let Some(buf) = &mut self.buf {
        let mut local = [0u8; 64];
        let len = self.bulk_out.read(&mut local).unwrap_or_default();
        if buf.len() + len <= buf.capacity() {
          buf.extend_from_slice(&local[..len]);
        }
        debug!("received {} for script", len);
      }
    }
  }
}

/// generated from flash unique id
fn get_serial_number() -> String {
  let mut buf = [0; 8];
  interrupt::free(|_| unsafe { flash_unique_id(&mut buf, true) });
  let mut str = String::with_capacity(16);
  write!(&mut str, "{:016X}", u64::from_ne_bytes(buf)).unwrap();
  str
}

/// 0xaabc - a is major, b is minor, c is patch
fn get_version_bcd() -> u16 {
  env!("CARGO_PKG_VERSION_MAJOR").parse::<u16>().unwrap() * 0x100
    + env!("CARGO_PKG_VERSION_MINOR").parse::<u16>().unwrap() * 0x10
    + env!("CARGO_PKG_VERSION_PATCH").parse::<u16>().unwrap()
}
