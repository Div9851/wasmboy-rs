use crate::console_log;
use crate::inst;
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

    pub fn dump_inst_table(&self) {
        let inst_table = inst::generate_inst_table();
        let prefix_inst_table = inst::generate_prefix_inst_table();
        console_log!("inst table");
        for (opcode, inst) in inst_table.into_iter().enumerate() {
            console_log!("{:x}: {:?}", opcode, inst);
        }
        console_log!("prefix inst table");
        for (opcode, inst) in prefix_inst_table.into_iter().enumerate() {
            console_log!("{:x}: {:?}", opcode, inst);
        }
    }
}
