use std::{env,fs};
use std::path::PathBuf;

fn main() {
  let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("memory.x");
  fs::copy("memory.x", out).unwrap();
  println!("cargo:rustc-link-search={}", out.display());
  println!("cargo:rerun-if-changed=memory.x");
}
