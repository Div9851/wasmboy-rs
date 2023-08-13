use crate::console_log;

pub struct Bus {
    pub cart_rom: [u8; 32 * 1024],
    pub cart_ram: [u8; 8 * 1024],
    pub work_ram: [u8; 8 * 1024],
    pub high_ram: [u8; 127],
    pub interrupt_enable_register: u8,
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            cart_rom: [0; 32 * 1024],
            cart_ram: [0; 8 * 1024],
            work_ram: [0; 8 * 1024],
            high_ram: [0; 127],
            interrupt_enable_register: 0,
        }
    }

    pub fn tick(&mut self) {
        // TODO: advance the processing of each component by 4 clock cycles.
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7fff => self.cart_rom[addr as usize],
            0xa000..=0xbfff => self.cart_ram[addr as usize - 0xa000],
            0xc000..=0xdfff => self.work_ram[addr as usize - 0xc000],
            0xff80..=0xfffe => self.high_ram[addr as usize - 0xff80],
            0xffff => self.interrupt_enable_register,
            _ => 0xff,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0xa000..=0xbfff => self.cart_ram[addr as usize - 0xa000] = value,
            0xc000..=0xdfff => self.work_ram[addr as usize - 0xc000] = value,
            0xff01 => console_log!("{}", value as char),
            0xff80..=0xfffe => self.high_ram[addr as usize - 0xff80] = value,
            0xffff => self.interrupt_enable_register = value,
            _ => {}
        };
    }
}
