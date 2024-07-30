#![no_std]
use core::mem;

pub const VENDOR_ID: u16 = 0x1209; // pid.codes
pub const VENDOR_NAME: &str = "gosher studios";
pub const PRODUCT_ID: u16 = 0xf19d; // todo
pub const PRODUCT_NAME: &str = "floppapad";
pub const BULK_OUT_ADDR: u8 = 1;

#[repr(u8)]
#[non_exhaustive]
#[derive(Copy, Clone)]
pub enum ControlType {
  PreScript, // length in request value
  Test,
}

impl From<u8> for ControlType {
  fn from(value: u8) -> Self {
    // safety: exhaustive enum
    unsafe { mem::transmute(value) }
  }
}
