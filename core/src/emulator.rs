extern crate console_error_panic_hook;
use crate::console_log;
use crate::cpu::CPU;
use crate::inst;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Emulator {
    cpu: CPU,
    clocks: isize,
}

#[wasm_bindgen]
impl Emulator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Emulator {
        Emulator {
            cpu: CPU::new(),
            clocks: 0,
        }
    }

    pub fn init(&mut self) {
        // https://gbdev.io/pandocs/Power_Up_Sequence.html#cpu-registers
        self.cpu.registers.a = 0x00;
        self.cpu.registers.b = 0x00;
        self.cpu.registers.c = 0x13;
        self.cpu.registers.d = 0x00;
        self.cpu.registers.e = 0xd8;
        self.cpu.registers.f = 0x80;
        self.cpu.registers.h = 0x01;
        self.cpu.registers.l = 0x4d;
        self.cpu.registers.pc = 0x100;
        self.cpu.registers.sp = 0xfffe;
    }

    pub fn load_rom(&mut self, rom_data: &[u8]) {
        let n = rom_data.len();
        self.cpu.bus.cart_rom[0..n].copy_from_slice(rom_data);
    }

    pub fn next_frame(&mut self) {
        console_error_panic_hook::set_once();
        self.clocks += 17556;
        while self.clocks > 0 {
            self.cpu.execute_inst();
            self.clocks -= self.cpu.tick_count;
        }
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
