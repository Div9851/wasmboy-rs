pub const Z_FLAG: u8 = 1 << 7;
pub const N_FLAG: u8 = 1 << 6;
pub const H_FLAG: u8 = 1 << 5;
pub const C_FLAG: u8 = 1 << 4;

pub const VBLANK_INTERRUPT: u8 = 1;
pub const LCD_STAT_INTERRUPT: u8 = 1 << 1;
pub const TIMER_INTERRUPT: u8 = 1 << 2;
pub const SERIAL_INTERRUPT: u8 = 1 << 3;
pub const JOYPAD_INTERRUPT: u8 = 1 << 4;

pub const INTERRUPT_HANDLER: [u16; 5] = [0x40, 0x48, 0x50, 0x58, 0x60];
