use std::time::Duration;
use floppapad_firmware::{VENDOR_ID, PRODUCT_ID, BULK_OUT_ADDR};

const TIMEOUT: Duration = Duration::from_secs(1);

fn main() {
  let device =
    rusb::open_device_with_vid_pid(VENDOR_ID, PRODUCT_ID).expect("could not find floppapad");
  println!("connected!");
  println!(
    "{:?}",
    device.write_bulk(BULK_OUT_ADDR, b"the quick brown fox", TIMEOUT)
  );
}
