use std::env;
use std::time::Duration;
use rusb::{DeviceHandle, GlobalContext, Direction, RequestType, Recipient};
use owo_colors::OwoColorize;
use floppapad_firmware::{ControlType, BULK_OUT_ADDR, PRODUCT_ID, VENDOR_ID};

const TIMEOUT: Duration = Duration::from_secs(1);

fn main() {
  if let Err(e) = run() {
    println!("{} {}", "error:".red().bold(), e);
  }
}

fn run() -> Result<(), &'static str> {
  match env::args().nth(1).as_deref() {
    Some("info") | Some("i") => {
      let dev = open_dev()?;
      let desc = dev
        .device()
        .device_descriptor()
        .map_err(|_| "could not get device descriptor")?;
      println!(
        "{} {} {}",
        dev
          .read_manufacturer_string_ascii(&desc)
          .map_err(|_| "failed reading strings")?
          .bold(),
        dev
          .read_product_string_ascii(&desc)
          .map_err(|_| "failed reading strings")?
          .bold(),
        format!("({:x}:{:x})", desc.vendor_id(), desc.product_id())
          .bright_white()
          .bold()
      );
      println!(
        "serial: {}",
        dev
          .read_serial_number_string_ascii(&desc)
          .map_err(|_| "failed reading strings")?
          .bright_white()
      );
      println!("firmware version: {}", desc.device_version().bright_white());
    }
    Some("up") => {
      let src = b"fn init() { print(\"meoww\"); }";
      let dev = open_dev()?;
      write_control(&dev, ControlType::PreScript, src.len() as _)?;
      for chunk in src.chunks(64) {
        dev
          .write_bulk(BULK_OUT_ADDR, chunk, TIMEOUT)
          .map_err(|_| "failed writing script")?;
      }
    }
    Some("test") => {
      let dev = open_dev()?;
      write_control(&dev, ControlType::Test, 0)?;
    }
    _ => {
      println!(
        "{}",
        format!("floppapad cli v{}", env!("CARGO_PKG_VERSION")).bold()
      );
      println!("commands:");
      println!("info - {}", "show connected device info".bright_white());
    }
  }
  Ok(())
}

fn write_control(
  dev: &DeviceHandle<GlobalContext>,
  ty: ControlType,
  val: u16,
) -> Result<(), &'static str> {
  dev
    .write_control(
      rusb::request_type(Direction::Out, RequestType::Vendor, Recipient::Interface),
      ty as _,
      val,
      0,
      &[],
      TIMEOUT,
    )
    .map_err(|_| "could not write control request")?;
  Ok(())
}

fn open_dev() -> Result<DeviceHandle<GlobalContext>, &'static str> {
  rusb::open_device_with_vid_pid(VENDOR_ID, PRODUCT_ID).ok_or("could not connect to floppapad")
}
