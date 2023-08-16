use crate::bus::Bus;
use crate::consts;
use crate::context::Context;
use crate::inst::{self, Inst, JumpCond, Op16, Op8};

pub struct CPU {
    pub bus: Bus,
    pub registers: Registers,
    pub ctx: Context,
    pub is_halt: bool,
    pub prev_ei: bool,
    pub interrupt_master_enable: bool,
    pub tick_count: isize,
    pub inst_table: [Inst; 256],
    pub prefix_inst_table: [Inst; 256],
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            bus: Bus::new(),
            registers: Registers::default(),
            ctx: Context::default(),
            is_halt: false,
            prev_ei: false,
            interrupt_master_enable: false,
            tick_count: 0,
            inst_table: inst::generate_inst_table(),
            prefix_inst_table: inst::generate_prefix_inst_table(),
        }
    }

    fn tick(&mut self) {
        self.tick_count += 1;
        self.bus.tick(&mut self.ctx);
    }

    fn read(&mut self, addr: u16) -> u8 {
        self.tick();
        self.bus.read(&mut self.ctx, addr)
    }

    fn write(&mut self, addr: u16, value: u8) {
        self.tick();
        self.bus.write(&mut self.ctx, addr, value);
    }

    fn get8(&mut self, src: Op8) -> u8 {
        match src {
            Op8::RegA => self.registers.a,
            Op8::RegB => self.registers.b,
            Op8::RegC => self.registers.c,
            Op8::RegD => self.registers.d,
            Op8::RegE => self.registers.e,
            Op8::RegH => self.registers.h,
            Op8::RegL => self.registers.l,
            Op8::Imm => self.fetch(),
            Op8::Addr(op) => {
                let addr = self.get16(op);
                self.read(addr)
            }
        }
    }

    fn set8(&mut self, dst: Op8, value: u8) {
        match dst {
            Op8::RegA => self.registers.a = value,
            Op8::RegB => self.registers.b = value,
            Op8::RegC => self.registers.c = value,
            Op8::RegD => self.registers.d = value,
            Op8::RegE => self.registers.e = value,
            Op8::RegH => self.registers.h = value,
            Op8::RegL => self.registers.l = value,
            Op8::Imm => unreachable!(),
            Op8::Addr(op) => {
                let addr = self.get16(op);
                self.write(addr, value);
            }
        }
    }

    fn get16(&mut self, src: Op16) -> u16 {
        match src {
            Op16::RegBC => self.registers.get_bc(),
            Op16::RegDE => self.registers.get_de(),
            Op16::RegHL => self.registers.get_hl(),
            Op16::RegSP => self.registers.sp,
            Op16::RegAF => self.registers.get_af(),
            Op16::Imm => {
                let lower = self.fetch() as u16;
                let upper = self.fetch() as u16;
                (upper << 8) | lower
            }
            Op16::AddImmToFF00 => {
                let offset = self.fetch() as u16;
                0xff00u16.wrapping_add(offset)
            }
            Op16::AddRegCToFF00 => {
                let offset = self.registers.c as u16;
                0xff00u16.wrapping_add(offset)
            }
            Op16::AddrImm | Op16::AddImmToSP => unreachable!(),
        }
    }

    fn set16(&mut self, dst: Op16, value: u16) {
        match dst {
            Op16::RegBC => self.registers.set_bc(value),
            Op16::RegDE => self.registers.set_de(value),
            Op16::RegHL => self.registers.set_hl(value),
            Op16::RegSP => self.registers.sp = value,
            Op16::RegAF => self.registers.set_af(value),
            Op16::Imm | Op16::AddImmToSP | Op16::AddImmToFF00 | Op16::AddRegCToFF00 => {
                unreachable!()
            }
            Op16::AddrImm => {
                let lower = self.fetch() as u16;
                let upper = self.fetch() as u16;
                let addr = (upper << 8) | lower;
                self.write(addr, (value & 0xff) as u8);
                self.write(addr + 1, (value >> 8) as u8);
            }
        }
    }

    fn is_jump_cond_satisfied(&self, cond: JumpCond) -> bool {
        match cond {
            JumpCond::NONE => true,
            JumpCond::NZ => !self.registers.get_z_flag(),
            JumpCond::Z => self.registers.get_z_flag(),
            JumpCond::NC => !self.registers.get_c_flag(),
            JumpCond::C => self.registers.get_c_flag(),
        }
    }

    fn fetch(&mut self) -> u8 {
        let value = self.read(self.registers.pc);
        self.registers.pc = self.registers.pc.wrapping_add(1);
        value
    }

    fn handle_interrupt(&mut self, interrupt: u8) {
        for i in 0..5 {
            if interrupt & (1 << i) != 0 {
                self.interrupt_master_enable = false;
                self.ctx.interrupt_flag ^= 1 << i;
                self.tick();
                self.tick();
                self.registers.sp = self.registers.sp.wrapping_sub(2);
                self.write(self.registers.sp, (self.registers.pc & 0xff) as u8);
                self.write(
                    self.registers.sp.wrapping_add(1),
                    (self.registers.pc >> 8) as u8,
                );
                self.tick();
                self.registers.pc = consts::INTERRUPT_HANDLER[i];
                break;
            }
        }
    }

    pub fn execute(&mut self) {
        let interrupt = self.ctx.interrupt_flag & self.ctx.interrupt_enable & 0x1f;
        if interrupt != 0 {
            self.is_halt = false;
            if self.interrupt_master_enable {
                self.handle_interrupt(interrupt);
                return;
            }
        }
        if self.is_halt {
            self.tick();
            return;
        }
        let inst = self.execute_inst();
        if self.prev_ei {
            self.interrupt_master_enable = true;
        }
        self.prev_ei = matches!(inst, Inst::EI);
    }

    fn execute_inst(&mut self) -> Inst {
        let opcode = self.fetch() as usize;
        let inst = self.inst_table[opcode];
        match inst {
            Inst::LD(dst, src) => {
                let value = self.get8(src);
                self.set8(dst, value);
            }
            Inst::LDI(Op8::Addr(Op16::RegHL), Op8::RegA) => {
                let hl = self.registers.get_hl();
                let value = self.registers.a;
                self.write(hl, value);
                self.registers.set_hl(hl.wrapping_add(1));
            }
            Inst::LDI(Op8::RegA, Op8::Addr(Op16::RegHL)) => {
                let hl = self.registers.get_hl();
                let value = self.read(hl);
                self.registers.a = value;
                self.registers.set_hl(hl.wrapping_add(1));
            }
            Inst::LDD(Op8::Addr(Op16::RegHL), Op8::RegA) => {
                let hl = self.registers.get_hl();
                let value = self.registers.a;
                self.write(hl, value);
                self.registers.set_hl(hl.wrapping_sub(1));
            }
            Inst::LDD(Op8::RegA, Op8::Addr(Op16::RegHL)) => {
                let hl = self.registers.get_hl();
                let value = self.read(hl);
                self.registers.a = value;
                self.registers.set_hl(hl.wrapping_sub(1));
            }
            Inst::LD16(Op16::AddrImm, Op16::RegSP) => {
                let value = self.registers.sp;
                self.set16(Op16::AddrImm, value);
            }
            Inst::LD16(Op16::RegSP, Op16::RegHL) => {
                self.tick();
                let value = self.registers.get_hl();
                self.registers.sp = value;
            }
            Inst::LD16(Op16::RegHL, Op16::AddImmToSP) => {
                let a = self.registers.sp;
                let b = (self.fetch() as i8) as u16; // sign extension
                self.tick();
                let value = a.wrapping_add(b);
                let carry = ((a & 0xff) + (b & 0xff)) > 0xff;
                let half_carry = ((a & 0xf) + (b & 0xf)) > 0xf;
                self.registers.set_hl(value);
                self.registers.set_z_flag(false);
                self.registers.set_n_flag(false);
                self.registers.set_h_flag(half_carry);
                self.registers.set_c_flag(carry);
            }
            Inst::LD16(dst, src) => {
                let value = self.get16(src);
                self.set16(dst, value);
            }
            Inst::PUSH(op) => {
                let value = self.get16(op);
                self.tick();
                self.registers.sp = self.registers.sp.wrapping_sub(2);
                self.write(self.registers.sp, (value & 0xff) as u8);
                self.write(self.registers.sp.wrapping_add(1), (value >> 8) as u8);
            }
            Inst::POP(op) => {
                let lower = self.read(self.registers.sp) as u16;
                let upper = self.read(self.registers.sp.wrapping_add(1)) as u16;
                self.registers.sp = self.registers.sp.wrapping_add(2);
                let value = (upper << 8) | lower;
                self.set16(op, value);
            }
            Inst::ADD(op) => {
                let a = self.registers.a;
                let b = self.get8(op);
                let (value, carry) = a.overflowing_add(b);
                let half_carry = ((a & 0xf) + (b & 0xf)) & 0x10 == 0x10;
                self.registers.a = value;
                self.registers.set_z_flag(value == 0);
                self.registers.set_n_flag(false);
                self.registers.set_h_flag(half_carry);
                self.registers.set_c_flag(carry);
            }
            Inst::ADC(op) => {
                let a = self.registers.a as usize;
                let b = self.get8(op) as usize;
                let c = if self.registers.get_c_flag() { 1 } else { 0 };
                let value = (a.wrapping_add(b + c) & 0xff) as u8;
                let carry = (a + b + c) > 0xff;
                let half_carry = ((a & 0xf) + (b & 0xf) + c) > 0xf;
                self.registers.a = value;
                self.registers.set_z_flag(value == 0);
                self.registers.set_n_flag(false);
                self.registers.set_h_flag(half_carry);
                self.registers.set_c_flag(carry);
            }
            Inst::SUB(op) => {
                let a = self.registers.a;
                let b = self.get8(op);
                let (value, carry) = a.overflowing_sub(b);
                let half_carry = (a & 0xf) < (b & 0xf);
                self.registers.a = value;
                self.registers.set_z_flag(value == 0);
                self.registers.set_n_flag(true);
                self.registers.set_h_flag(half_carry);
                self.registers.set_c_flag(carry);
            }
            Inst::SBC(op) => {
                let a = self.registers.a as usize;
                let b = self.get8(op) as usize;
                let c = if self.registers.get_c_flag() { 1 } else { 0 };
                let value = (a.wrapping_sub(b + c) & 0xff) as u8;
                let carry = a < (b + c);
                let half_carry = (a & 0xf) < (b & 0xf) + c;
                self.registers.a = value;
                self.registers.set_z_flag(value == 0);
                self.registers.set_n_flag(true);
                self.registers.set_h_flag(half_carry);
                self.registers.set_c_flag(carry);
            }
            Inst::AND(op) => {
                let a = self.registers.a;
                let b = self.get8(op);
                let value = a & b;
                self.registers.a = value;
                self.registers.set_z_flag(value == 0);
                self.registers.set_n_flag(false);
                self.registers.set_h_flag(true);
                self.registers.set_c_flag(false);
            }
            Inst::OR(op) => {
                let a = self.registers.a;
                let b = self.get8(op);
                let value = a | b;
                self.registers.a = value;
                self.registers.set_z_flag(value == 0);
                self.registers.set_n_flag(false);
                self.registers.set_h_flag(false);
                self.registers.set_c_flag(false);
            }
            Inst::XOR(op) => {
                let a = self.registers.a;
                let b = self.get8(op);
                let value = a ^ b;
                self.registers.a = value;
                self.registers.set_z_flag(value == 0);
                self.registers.set_n_flag(false);
                self.registers.set_h_flag(false);
                self.registers.set_c_flag(false);
            }
            Inst::CP(op) => {
                let a = self.registers.a;
                let b = self.get8(op);
                let (value, carry) = a.overflowing_sub(b);
                let half_carry = (a & 0xf) < (b & 0xf);
                self.registers.set_z_flag(value == 0);
                self.registers.set_n_flag(true);
                self.registers.set_h_flag(half_carry);
                self.registers.set_c_flag(carry);
            }
            Inst::INC(op) => {
                let a = self.get8(op);
                let value = a.wrapping_add(1);
                let half_carry = ((a & 0xf) + 1) & 0x10 == 0x10;
                self.set8(op, value);
                self.registers.set_z_flag(value == 0);
                self.registers.set_n_flag(false);
                self.registers.set_h_flag(half_carry);
            }
            Inst::DEC(op) => {
                let a = self.get8(op);
                let value = a.wrapping_sub(1);
                let half_carry = (a & 0xf) < 1;
                self.set8(op, value);
                self.registers.set_z_flag(value == 0);
                self.registers.set_n_flag(true);
                self.registers.set_h_flag(half_carry);
            }
            Inst::DAA => {
                //ref: https://ehaskins.com/2018-01-30%20Z80%20DAA/
                let a = self.registers.a;
                let mut correction = 0;
                let mut carry = false;
                if self.registers.get_h_flag() || (!self.registers.get_n_flag() && (a & 0xf) > 0x9)
                {
                    correction += 0x6;
                }
                if self.registers.get_c_flag() || (!self.registers.get_n_flag() && a > 0x99) {
                    correction += 0x60;
                    carry = true;
                }
                let value = if self.registers.get_n_flag() {
                    self.registers.a.wrapping_sub(correction)
                } else {
                    self.registers.a.wrapping_add(correction)
                };
                self.registers.a = value;
                self.registers.set_z_flag(value == 0);
                self.registers.set_h_flag(false);
                self.registers.set_c_flag(carry);
            }
            Inst::CPL => {
                self.registers.a ^= 0xff;
                self.registers.set_n_flag(true);
                self.registers.set_h_flag(true);
            }
            Inst::ADDHL(op) => {
                self.tick();
                let a = self.registers.get_hl();
                let b = self.get16(op);
                let (value, carry) = a.overflowing_add(b);
                let half_carry = ((a & 0xfff) + (b & 0xfff)) > 0xfff;
                self.registers.set_hl(value);
                self.registers.set_n_flag(false);
                self.registers.set_h_flag(half_carry);
                self.registers.set_c_flag(carry);
            }
            Inst::INC16(op) => {
                let a = self.get16(op);
                let value = a.wrapping_add(1);
                self.tick();
                self.set16(op, value);
            }
            Inst::DEC16(op) => {
                let a = self.get16(op);
                let value = a.wrapping_sub(1);
                self.tick();
                self.set16(op, value);
            }
            Inst::ADDSP => {
                let a = self.registers.sp;
                let b = (self.fetch() as i8) as u16; // sign extension
                self.tick();
                let value = a.wrapping_add(b);
                let carry = ((a & 0xff) + (b & 0xff)) > 0xff;
                let half_carry = ((a & 0xf) + (b & 0xf)) > 0xf;
                self.tick();
                self.registers.sp = value;
                self.registers.set_z_flag(false);
                self.registers.set_n_flag(false);
                self.registers.set_h_flag(half_carry);
                self.registers.set_c_flag(carry);
            }
            Inst::RLCA => {
                let a = self.registers.a;
                let value = ((a & 0x7f) << 1) + ((a >> 7) & 1);
                let carry = (a & (1 << 7)) != 0;
                self.registers.a = value;
                self.registers.set_z_flag(false);
                self.registers.set_n_flag(false);
                self.registers.set_h_flag(false);
                self.registers.set_c_flag(carry);
            }
            Inst::RLA => {
                let a = self.registers.a;
                let c = if self.registers.get_c_flag() { 1 } else { 0 };
                let value = ((a & 0x7f) << 1) + c;
                let carry = (a & (1 << 7)) != 0;
                self.registers.a = value;
                self.registers.set_z_flag(false);
                self.registers.set_n_flag(false);
                self.registers.set_h_flag(false);
                self.registers.set_c_flag(carry);
            }
            Inst::RRCA => {
                let a = self.registers.a;
                let value = ((a & 0xfe) >> 1) + ((a & 1) << 7);
                let carry = (a & 1) != 0;
                self.registers.a = value;
                self.registers.set_z_flag(false);
                self.registers.set_n_flag(false);
                self.registers.set_h_flag(false);
                self.registers.set_c_flag(carry);
            }
            Inst::RRA => {
                let a = self.registers.a;
                let c = if self.registers.get_c_flag() { 1 } else { 0 };
                let value = ((a & 0xfe) >> 1) + (c << 7);
                let carry = (a & 1) != 0;
                self.registers.a = value;
                self.registers.set_z_flag(false);
                self.registers.set_n_flag(false);
                self.registers.set_h_flag(false);
                self.registers.set_c_flag(carry);
            }
            Inst::CCF => {
                self.registers.set_n_flag(false);
                self.registers.set_h_flag(false);
                self.registers.set_c_flag(!self.registers.get_c_flag());
            }
            Inst::SCF => {
                self.registers.set_n_flag(false);
                self.registers.set_h_flag(false);
                self.registers.set_c_flag(true);
            }
            Inst::NOP => {}
            Inst::HALT => {
                self.is_halt = true;
            }
            Inst::STOP => {}
            Inst::DI => {
                self.interrupt_master_enable = false;
            }
            Inst::EI => {}
            Inst::JP(JumpCond::NONE, Op16::RegHL) => {
                self.registers.pc = self.registers.get_hl();
            }
            Inst::JP(cond, op) => {
                let addr = self.get16(op);
                if self.is_jump_cond_satisfied(cond) {
                    self.tick();
                    self.registers.pc = addr;
                }
            }
            Inst::JR(cond) => {
                let offset = (self.fetch() as i8) as u16;
                if self.is_jump_cond_satisfied(cond) {
                    self.tick();
                    self.registers.pc = self.registers.pc.wrapping_add(offset);
                }
            }
            Inst::CALL(cond) => {
                let lower = self.fetch() as u16;
                let upper = self.fetch() as u16;
                let addr = (upper << 8) | lower;
                if self.is_jump_cond_satisfied(cond) {
                    self.registers.sp = self.registers.sp.wrapping_sub(2);
                    self.write(self.registers.sp, (self.registers.pc & 0xff) as u8);
                    self.write(
                        self.registers.sp.wrapping_add(1),
                        (self.registers.pc >> 8) as u8,
                    );
                    self.tick();
                    self.registers.pc = addr;
                }
            }
            Inst::RET(cond) => {
                if !matches!(cond, JumpCond::NONE) {
                    self.tick();
                }
                if self.is_jump_cond_satisfied(cond) {
                    let lower = self.read(self.registers.sp) as u16;
                    let upper = self.read(self.registers.sp.wrapping_add(1)) as u16;
                    let addr = (upper << 8) | lower;
                    self.tick();
                    self.registers.pc = addr;
                    self.registers.sp = self.registers.sp.wrapping_add(2);
                }
            }
            Inst::RETI => {
                let lower = self.read(self.registers.sp) as u16;
                let upper = self.read(self.registers.sp.wrapping_add(1)) as u16;
                let addr = (upper << 8) | lower;
                self.tick();
                self.registers.pc = addr;
                self.registers.sp = self.registers.sp.wrapping_add(2);
                self.interrupt_master_enable = true;
            }
            Inst::RST(addr) => {
                self.registers.sp = self.registers.sp.wrapping_sub(2);
                self.write(self.registers.sp, (self.registers.pc & 0xff) as u8);
                self.write(
                    self.registers.sp.wrapping_add(1),
                    (self.registers.pc >> 8) as u8,
                );
                self.tick();
                self.registers.pc = addr;
            }
            Inst::PREFIX => {
                return self.execute_prefix_inst();
            }
            _ => {
                unreachable!("unknown opcode: {:x}", opcode);
            }
        }
        inst
    }

    fn execute_prefix_inst(&mut self) -> Inst {
        let opcode = self.fetch() as usize;
        let inst = self.prefix_inst_table[opcode];
        match inst {
            Inst::RLC(op) => {
                let a = self.get8(op);
                let value = ((a & 0x7f) << 1) + ((a >> 7) & 1);
                let carry = (a & (1 << 7)) != 0;
                self.set8(op, value);
                self.registers.set_z_flag(value == 0);
                self.registers.set_n_flag(false);
                self.registers.set_h_flag(false);
                self.registers.set_c_flag(carry);
            }
            Inst::RL(op) => {
                let a = self.get8(op);
                let c = if self.registers.get_c_flag() { 1 } else { 0 };
                let value = ((a & 0x7f) << 1) + c;
                let carry = (a & (1 << 7)) != 0;
                self.set8(op, value);
                self.registers.set_z_flag(value == 0);
                self.registers.set_n_flag(false);
                self.registers.set_h_flag(false);
                self.registers.set_c_flag(carry);
            }
            Inst::RRC(op) => {
                let a = self.get8(op);
                let value = ((a & 0xfe) >> 1) + ((a & 1) << 7);
                let carry = (a & 1) != 0;
                self.set8(op, value);
                self.registers.set_z_flag(value == 0);
                self.registers.set_n_flag(false);
                self.registers.set_h_flag(false);
                self.registers.set_c_flag(carry);
            }
            Inst::RR(op) => {
                let a = self.get8(op);
                let c = if self.registers.get_c_flag() { 1 } else { 0 };
                let value = ((a & 0xfe) >> 1) + (c << 7);
                let carry = (a & 1) != 0;
                self.set8(op, value);
                self.registers.set_z_flag(value == 0);
                self.registers.set_n_flag(false);
                self.registers.set_h_flag(false);
                self.registers.set_c_flag(carry);
            }
            Inst::SLA(op) => {
                let a = self.get8(op);
                let value = (a & 0x7f) << 1;
                let carry = (a & (1 << 7)) != 0;
                self.set8(op, value);
                self.registers.set_z_flag(value == 0);
                self.registers.set_n_flag(false);
                self.registers.set_h_flag(false);
                self.registers.set_c_flag(carry);
            }
            Inst::SWAP(op) => {
                let a = self.get8(op);
                let value = ((a & 0xf0) >> 4) + ((a & 0xf) << 4);
                self.set8(op, value);
                self.registers.set_z_flag(value == 0);
                self.registers.set_n_flag(false);
                self.registers.set_h_flag(false);
                self.registers.set_c_flag(false);
            }
            Inst::SRA(op) => {
                let a = self.get8(op);
                let msb = a & (1 << 7);
                let value = ((a & 0xfe) >> 1) + msb;
                let carry = (a & 1) != 0;
                self.set8(op, value);
                self.registers.set_z_flag(value == 0);
                self.registers.set_n_flag(false);
                self.registers.set_h_flag(false);
                self.registers.set_c_flag(carry);
            }
            Inst::SRL(op) => {
                let a = self.get8(op);
                let value = (a & 0xfe) >> 1;
                let carry = (a & 1) != 0;
                self.set8(op, value);
                self.registers.set_z_flag(value == 0);
                self.registers.set_n_flag(false);
                self.registers.set_h_flag(false);
                self.registers.set_c_flag(carry);
            }
            Inst::BIT(n, op) => {
                let a = self.get8(op);
                self.registers.set_z_flag(((a >> n) & 1) == 0);
                self.registers.set_n_flag(false);
                self.registers.set_h_flag(true);
            }
            Inst::SET(n, op) => {
                let a = self.get8(op);
                let value = a | (1 << n);
                self.set8(op, value);
            }
            Inst::RES(n, op) => {
                let a = self.get8(op);
                let value = a & (0xff ^ (1 << n));
                self.set8(op, value);
            }
            _ => unreachable!("unknown opcode: {:x}", opcode),
        }
        inst
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: u8, // only upper 4 bits are used and lower 4 bits should be zero.
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
}

impl Registers {
    pub fn get_af(&self) -> u16 {
        return ((self.a as u16) << 8) | self.f as u16;
    }

    pub fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.f = (value & 0xf0) as u8;
    }

    pub fn get_bc(&self) -> u16 {
        return ((self.b as u16) << 8) | self.c as u16;
    }

    pub fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = (value & 0xff) as u8;
    }

    pub fn get_de(&self) -> u16 {
        return ((self.d as u16) << 8) | self.e as u16;
    }

    pub fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = (value & 0xff) as u8;
    }

    pub fn get_hl(&self) -> u16 {
        return ((self.h as u16) << 8) | self.l as u16;
    }

    pub fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = (value & 0xff) as u8;
    }

    pub fn get_z_flag(&self) -> bool {
        (self.f & consts::Z_FLAG) != 0
    }

    pub fn set_z_flag(&mut self, value: bool) {
        if value {
            self.f |= consts::Z_FLAG;
        } else {
            self.f &= 0xff ^ consts::Z_FLAG;
        }
    }

    pub fn get_n_flag(&self) -> bool {
        (self.f & consts::N_FLAG) != 0
    }

    pub fn set_n_flag(&mut self, value: bool) {
        if value {
            self.f |= consts::N_FLAG;
        } else {
            self.f &= 0xff ^ consts::N_FLAG;
        }
    }

    pub fn get_h_flag(&self) -> bool {
        (self.f & consts::H_FLAG) != 0
    }

    pub fn set_h_flag(&mut self, value: bool) {
        if value {
            self.f |= consts::H_FLAG;
        } else {
            self.f &= 0xff ^ consts::H_FLAG;
        }
    }

    pub fn get_c_flag(&self) -> bool {
        (self.f & consts::C_FLAG) != 0
    }

    pub fn set_c_flag(&mut self, value: bool) {
        if value {
            self.f |= consts::C_FLAG;
        } else {
            self.f &= 0xff ^ consts::C_FLAG;
        }
    }
}
