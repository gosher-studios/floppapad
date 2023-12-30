use dioxus::prelude::*;
use web_sys::{Hid, HidDeviceRequestOptions};
use js_sys::Array;
use log::LevelFilter;

fn main() {
  dioxus_logger::init(LevelFilter::Debug);
  dioxus_web::launch(App);
}

#[component]
fn App(cx: Scope) -> Element {
  let win = web_sys::window().unwrap();
  let hid = win.navigator().hid();
  if hid.is_undefined() {
    cx.render(rsx! { h1 { class: "text-5xl", "webhid unsupported" } })
  } else {
    cx.render(rsx! { h1 { class: "text-5xl", "hell yeah" }
      button { onclick: move |_ | {hid.request_device(&HidDeviceRequestOptions::new(&Array::new()));}, "test"} })
  }
}
