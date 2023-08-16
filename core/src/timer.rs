use crate::consts;
use crate::context::Context;

// TIMA is incremented at the clock frequency specified by the TAC register.
// DIV is incremented at a rate of 16384 Hz (= CPU Clock / 256)
#[derive(Default)]
pub struct Timer {
    pub div: u8,
    pub tima: u8,
    pub tma: u8,
    pub tac: u8,
    pub timer_counter: usize,
    pub divider_counter: usize,
}

impl Timer {
    pub fn tick(&mut self, ctx: &mut Context) {
        self.divider_counter += 1;
        if self.divider_counter == 256 {
            self.div = self.div.wrapping_add(1);
            self.divider_counter = 0;
        }
        if self.tac & (1 << 2) != 0 {
            let mode = self.tac & 3;
            let target = match mode {
                0 => 1024, // 4096 Hz (= CPU Clock / 1024)
                1 => 16,   // 262144 Hz (= CPU Clock / 16)
                2 => 64,   // 65536 Hz (= CPU Clock / 64)
                3 => 256,  // 16384 Hz (=CPU Clock / 256)
                _ => unreachable!(),
            };
            self.timer_counter += 1;
            if self.timer_counter == target {
                if self.tima == 0xff {
                    self.tima = self.tma;
                    ctx.interrupt_flag |= consts::TIMER_INTERRUPT;
                } else {
                    self.tima += 1;
                }
                self.timer_counter = 0;
            }
        }
    }
}
