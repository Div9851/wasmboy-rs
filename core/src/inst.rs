#[derive(Debug, Clone, Copy)]
pub enum Op8 {
    RegA,
    RegB,
    RegC,
    RegD,
    RegE,
    RegH,
    RegL,
    Imm,
    Addr(Op16),
}

#[derive(Debug, Clone, Copy)]
pub enum Op16 {
    RegBC,
    RegDE,
    RegHL,
    RegSP,
    RegAF,
    Imm,           // nn
    AddImmToSP,    // SP + d
    AddImmToFF00,  // 0xff00 + n
    AddRegCToFF00, // 0xff00 + C
    AddrImm,       // (nn)
}

#[derive(Debug, Clone, Copy)]
pub enum JumpCond {
    NONE,
    NZ,
    Z,
    NC,
    C,
}

const OP8: [Op8; 8] = [
    Op8::RegB,
    Op8::RegC,
    Op8::RegD,
    Op8::RegE,
    Op8::RegH,
    Op8::RegL,
    Op8::Addr(Op16::RegHL),
    Op8::RegA,
];

const OP16: [Op16; 4] = [Op16::RegBC, Op16::RegDE, Op16::RegHL, Op16::RegSP];

const CONDITIONS: [JumpCond; 4] = [JumpCond::NZ, JumpCond::Z, JumpCond::NC, JumpCond::C];

#[derive(Debug, Clone, Copy)]
pub enum Inst {
    UNK,
    LD(Op8, Op8),
    LDI(Op8, Op8),
    LDD(Op8, Op8),
    LD16(Op16, Op16),
    PUSH(Op16),
    POP(Op16),
    ADD(Op8),
    ADC(Op8),
    SUB(Op8),
    SBC(Op8),
    AND(Op8),
    XOR(Op8),
    OR(Op8),
    CP(Op8),
    INC(Op8),
    DEC(Op8),
    DAA,
    CPL,
    ADDHL(Op16),
    INC16(Op16),
    DEC16(Op16),
    ADDSP,
    RLCA,
    RLA,
    RRCA,
    RRA,
    CCF,
    SCF,
    NOP,
    HALT,
    STOP,
    DI,
    EI,
    JP(JumpCond, Op16),
    JR(JumpCond),
    CALL(JumpCond),
    RET(JumpCond),
    RETI,
    RST(u16),
    PREFIX,
    RLC(Op8),
    RRC(Op8),
    RL(Op8),
    RR(Op8),
    SLA(Op8),
    SRA(Op8),
    SWAP(Op8),
    SRL(Op8),
    BIT(usize, Op8),
    RES(usize, Op8),
    SET(usize, Op8),
}

pub fn generate_inst_table() -> [Inst; 256] {
    let mut table = [Inst::UNK; 256];
    // 8-bit load inst
    {
        for (i, dst) in OP8.into_iter().enumerate() {
            for (j, src) in OP8.into_iter().enumerate() {
                let opcode = 0x40 + (i << 3) + j;
                table[opcode] = Inst::LD(dst, src);
            }
        }
        for (i, dst) in OP8.into_iter().enumerate() {
            let opcode = 0x06 + (i << 3);
            table[opcode] = Inst::LD(dst, Op8::Imm);
        }
        table[0x0a] = Inst::LD(Op8::RegA, Op8::Addr(Op16::RegBC));
        table[0x1a] = Inst::LD(Op8::RegA, Op8::Addr(Op16::RegDE));
        table[0xfa] = Inst::LD(Op8::RegA, Op8::Addr(Op16::Imm));
        table[0x02] = Inst::LD(Op8::Addr(Op16::RegBC), Op8::RegA);
        table[0x12] = Inst::LD(Op8::Addr(Op16::RegDE), Op8::RegA);
        table[0xea] = Inst::LD(Op8::Addr(Op16::Imm), Op8::RegA);
        table[0xf0] = Inst::LD(Op8::RegA, Op8::Addr(Op16::AddImmToFF00));
        table[0xe0] = Inst::LD(Op8::Addr(Op16::AddImmToFF00), Op8::RegA);
        table[0xf2] = Inst::LD(Op8::RegA, Op8::Addr(Op16::AddRegCToFF00));
        table[0xe2] = Inst::LD(Op8::Addr(Op16::AddRegCToFF00), Op8::RegA);
        table[0x22] = Inst::LDI(Op8::Addr(Op16::RegHL), Op8::RegA);
        table[0x2a] = Inst::LDI(Op8::RegA, Op8::Addr(Op16::RegHL));
        table[0x32] = Inst::LDD(Op8::Addr(Op16::RegHL), Op8::RegA);
        table[0x3a] = Inst::LDD(Op8::RegA, Op8::Addr(Op16::RegHL));
    }
    // 16-bit load inst
    {
        for (i, dst) in OP16.into_iter().enumerate() {
            let opcode = 1 + (i << 4);
            table[opcode] = Inst::LD16(dst, Op16::Imm);
        }
        table[0x08] = Inst::LD16(Op16::AddrImm, Op16::RegSP);
        table[0xf9] = Inst::LD16(Op16::RegSP, Op16::RegHL);
        for (i, op) in OP16.into_iter().enumerate() {
            let opcode = 0xc5 + (i << 4);
            if matches!(op, Op16::RegSP) {
                // PUSH SP does not exist.
                // This opcode corresponds to PUSH AF.
                table[opcode] = Inst::PUSH(Op16::RegAF);
            } else {
                table[opcode] = Inst::PUSH(op);
            }
        }
        for (i, op) in OP16.into_iter().enumerate() {
            let opcode = 0xc1 + (i << 4);
            if matches!(op, Op16::RegSP) {
                // POP SP does not exist.
                // This opcode corresponds to POP AF.
                table[opcode] = Inst::POP(Op16::RegAF);
            } else {
                table[opcode] = Inst::POP(op);
            }
        }
    }
    // 8-bit arithmetic/logic inst
    {
        for (i, op) in OP8.into_iter().enumerate() {
            let opcode = 0x80 + i;
            table[opcode] = Inst::ADD(op);
        }
        table[0xc6] = Inst::ADD(Op8::Imm);
        for (i, op) in OP8.into_iter().enumerate() {
            let opcode = 0x88 + i;
            table[opcode] = Inst::ADC(op);
        }
        table[0xce] = Inst::ADC(Op8::Imm);
        for (i, op) in OP8.into_iter().enumerate() {
            let opcode = 0x90 + i;
            table[opcode] = Inst::SUB(op);
        }
        table[0xd6] = Inst::SUB(Op8::Imm);
        for (i, op) in OP8.into_iter().enumerate() {
            let opcode = 0x98 + i;
            table[opcode] = Inst::SBC(op);
        }
        table[0xde] = Inst::SBC(Op8::Imm);
        for (i, op) in OP8.into_iter().enumerate() {
            let opcode = 0xa0 + i;
            table[opcode] = Inst::AND(op);
        }
        table[0xe6] = Inst::AND(Op8::Imm);
        for (i, op) in OP8.into_iter().enumerate() {
            let opcode = 0xa8 + i;
            table[opcode] = Inst::XOR(op);
        }
        table[0xee] = Inst::XOR(Op8::Imm);
        for (i, op) in OP8.into_iter().enumerate() {
            let opcode = 0xb0 + i;
            table[opcode] = Inst::OR(op);
        }
        table[0xf6] = Inst::OR(Op8::Imm);
        for (i, op) in OP8.into_iter().enumerate() {
            let opcode = 0xb8 + i;
            table[opcode] = Inst::CP(op);
        }
        table[0xfe] = Inst::CP(Op8::Imm);
        for (i, op) in OP8.into_iter().enumerate() {
            let opcode = 0x04 + (i << 3);
            table[opcode] = Inst::INC(op);
        }
        for (i, op) in OP8.into_iter().enumerate() {
            let opcode = 0x05 + (i << 3);
            table[opcode] = Inst::DEC(op);
        }
        table[0x27] = Inst::DAA;
        table[0x2f] = Inst::CPL;
    }
    // 16-bit arithmetic/logic inst
    {
        for (i, op) in OP16.into_iter().enumerate() {
            let opcode = 0x09 + (i << 4);
            table[opcode] = Inst::ADDHL(op);
        }
        for (i, op) in OP16.into_iter().enumerate() {
            let opcode = 0x03 + (i << 4);
            table[opcode] = Inst::INC16(op);
        }
        for (i, op) in OP16.into_iter().enumerate() {
            let opcode = 0x0b + (i << 4);
            table[opcode] = Inst::DEC16(op);
        }
        table[0xe8] = Inst::ADDSP;
        table[0xf8] = Inst::LD16(Op16::RegHL, Op16::AddImmToSP);
    }
    // rotate and shift inst
    {
        table[0x07] = Inst::RLCA;
        table[0x17] = Inst::RLA;
        table[0x0f] = Inst::RRCA;
        table[0x1f] = Inst::RRA;
    }
    // CPU control inst
    {
        table[0x3f] = Inst::CCF;
        table[0x37] = Inst::SCF;
        table[0x00] = Inst::NOP;
        table[0x76] = Inst::HALT;
        table[0x10] = Inst::STOP;
        table[0xf3] = Inst::DI;
        table[0xfb] = Inst::EI;
    }
    // jump inst
    {
        table[0xc3] = Inst::JP(JumpCond::NONE, Op16::Imm);
        table[0xe9] = Inst::JP(JumpCond::NONE, Op16::RegHL);
        for (i, cond) in CONDITIONS.into_iter().enumerate() {
            let opcode = 0xc2 + (i << 3);
            table[opcode] = Inst::JP(cond, Op16::Imm);
        }
        table[0x18] = Inst::JR(JumpCond::NONE);
        for (i, cond) in CONDITIONS.into_iter().enumerate() {
            let opcode = 0x20 + (i << 3);
            table[opcode] = Inst::JR(cond);
        }
        table[0xcd] = Inst::CALL(JumpCond::NONE);
        for (i, cond) in CONDITIONS.into_iter().enumerate() {
            let opcode = 0xc4 + (i << 3);
            table[opcode] = Inst::CALL(cond);
        }
        table[0xc9] = Inst::RET(JumpCond::NONE);
        for (i, cond) in CONDITIONS.into_iter().enumerate() {
            let opcode = 0xc0 + (i << 3);
            table[opcode] = Inst::RET(cond);
        }
        table[0xd9] = Inst::RETI;
        for i in 0..8 {
            let opcode = 0xc7 + (i << 3);
            table[opcode] = Inst::RST((i << 3) as u16);
        }
        table[0xcb] = Inst::PREFIX;
    }
    table
}

pub fn generate_prefix_inst_table() -> [Inst; 256] {
    let mut table = [Inst::UNK; 256];
    // rotate and shift inst
    {
        for (i, op) in OP8.into_iter().enumerate() {
            let opcode = i;
            table[opcode] = Inst::RLC(op);
        }
        for (i, op) in OP8.into_iter().enumerate() {
            let opcode = 0x08 + i;
            table[opcode] = Inst::RRC(op);
        }
        for (i, op) in OP8.into_iter().enumerate() {
            let opcode = 0x10 + i;
            table[opcode] = Inst::RL(op);
        }
        for (i, op) in OP8.into_iter().enumerate() {
            let opcode = 0x18 + i;
            table[opcode] = Inst::RR(op);
        }
        for (i, op) in OP8.into_iter().enumerate() {
            let opcode = 0x20 + i;
            table[opcode] = Inst::SLA(op);
        }
        for (i, op) in OP8.into_iter().enumerate() {
            let opcode = 0x28 + i;
            table[opcode] = Inst::SRA(op);
        }
        for (i, op) in OP8.into_iter().enumerate() {
            let opcode = 0x30 + i;
            table[opcode] = Inst::SWAP(op);
        }
        for (i, op) in OP8.into_iter().enumerate() {
            let opcode = 0x38 + i;
            table[opcode] = Inst::SRL(op);
        }
        // single-bit operation inst
        {
            for n in 0..8 {
                for (i, op) in OP8.into_iter().enumerate() {
                    let opcode = 0x40 + (n << 3) + i;
                    table[opcode] = Inst::BIT(n, op);
                }
            }
            for n in 0..8 {
                for (i, op) in OP8.into_iter().enumerate() {
                    let opcode = 0x80 + (n << 3) + i;
                    table[opcode] = Inst::RES(n, op);
                }
            }
            for n in 0..8 {
                for (i, op) in OP8.into_iter().enumerate() {
                    let opcode = 0xc0 + (n << 3) + i;
                    table[opcode] = Inst::SET(n, op);
                }
            }
        }
    }
    table
}
