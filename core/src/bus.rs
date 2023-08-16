use crate::console_log;
use crate::context::Context;
use crate::timer::Timer;

pub struct Bus {
    pub cart_rom: [u8; 32 * 1024],
    pub cart_ram: [u8; 8 * 1024],
    pub work_ram: [u8; 8 * 1024],
    pub high_ram: [u8; 127],
    pub timer: Timer,
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            cart_rom: [0; 32 * 1024],
            cart_ram: [0; 8 * 1024],
            work_ram: [0; 8 * 1024],
            high_ram: [0; 127],
            timer: Timer::default(),
        }
    }

    pub fn tick(&mut self, ctx: &mut Context) {
        // advance the processing of each component by 4 clock cycles.
        for _ in 0..4 {
            self.timer.tick(ctx);
        }
    }

    pub fn read(&mut self, ctx: &Context, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7fff => self.cart_rom[addr as usize],
            0xa000..=0xbfff => self.cart_ram[addr as usize - 0xa000],
            0xc000..=0xdfff => self.work_ram[addr as usize - 0xc000],
            0xff04 => self.timer.div,
            0xff05 => self.timer.tima,
            0xff06 => self.timer.tma,
            0xff07 => self.timer.tac,
            0xff0f => ctx.interrupt_flag,
            0xff80..=0xfffe => self.high_ram[addr as usize - 0xff80],
            0xffff => ctx.interrupt_enable,
            _ => 0xff,
        }
    }

    pub fn write(&mut self, ctx: &mut Context, addr: u16, value: u8) {
        match addr {
            0xa000..=0xbfff => self.cart_ram[addr as usize - 0xa000] = value,
            0xc000..=0xdfff => self.work_ram[addr as usize - 0xc000] = value,
            0xff01 => console_log!("{}", value as char),
            0xff04 => self.timer.div = 0, // Writing any value to this register resets it to 0x00.
            0xff05 => self.timer.tima = value,
            0xff06 => self.timer.tma = value,
            0xff07 => self.timer.tac = value,
            0xff0f => ctx.interrupt_flag = value,
            0xff80..=0xfffe => self.high_ram[addr as usize - 0xff80] = value,
            0xffff => ctx.interrupt_enable = value,
            _ => {}
        };
    }
}
