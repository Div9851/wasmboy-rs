use crate::console_log;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Emulator {}

#[wasm_bindgen]
impl Emulator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Emulator {
        Emulator {}
    }

    pub fn greet(&self, name: &str) -> String {
        console_log!("Hello, {}!", name);
        format!("Hello, {}!", name)
    }
}
