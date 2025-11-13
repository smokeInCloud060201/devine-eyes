use leptos::mount::mount_to_body;
use wasm_bindgen::prelude::*;

mod app;
mod components;
mod pages;
mod services;
mod utils;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(app::App);
}

