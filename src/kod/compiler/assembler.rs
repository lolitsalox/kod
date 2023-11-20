
use capstone::prelude::*;

pub fn disassemble_machine_code(machine_code: &[u8]) {
    // Create a Capstone engine
    let cs = Capstone::new()
        .x86()
        .mode(arch::x86::ArchMode::Mode64)
        .build()
        .expect("Failed to create Capstone engine");

    // Disassemble the machine code
    if let Ok(insns) = cs.disasm_all(machine_code, 0x0) {
        for insn in insns.iter() {
            println!("{:x}:\t{}\t{}", insn.address(), insn.mnemonic().unwrap(), insn.op_str().unwrap());
        }
    } else {
        println!("Failed to disassemble machine code");
    };

}

pub struct Assembler {
    pub machine_code: Vec<u8>,
}

pub struct Label {
    offset: Option<usize>,
    jump_slots: Vec<usize>,
}

pub struct ModRm {
    pub raw: u8,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ModRmEnum {
    Mem         = 0b00,
    MemDisp8    = 0b01,
    MemDisp32   = 0b10,
    Reg         = 0b11,
}

impl ModRmEnum {
    pub fn encode(&self) -> u8 {
        *self as u8
    }
}

impl ModRm {
    pub fn new(rm: u8, reg: u8, mode: u8) -> Self {
        ModRm { 
            raw: (mode & 0b11) << 6 | (reg & 0b111) << 3 | (rm & 0b111)
        }
    }

    pub fn encode(&self) -> u8 {
        self.raw
    }

    pub fn set_mode(&mut self, mode: u8) {
        self.raw = (mode & 0b11) << 6 | (self.raw & !(0b11 << 6));
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Rex {
    pub raw: u8,
    /*
    u8 B : 1  // ModRM::RM
    u8 X : 1; // SIB::Index
    u8 R : 1; // ModRM::Reg
    u8 W : 1; // Operand size override
    u8 _ : 4; // { 0b0100 }
    */
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Patchable {
    No = 0,
    Yes = 1,
}

impl Patchable {
    pub fn encode(&self) -> u8 {
        *self as u8
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum RexW {
    No = 0,
    Yes = 1,
}

impl RexW {
    pub fn encode(&self) -> u8 {
        *self as u8
    }
}

impl Rex {
    pub fn new(b: u8, x: u8, r: u8, w: u8) -> Self {
        Rex { 
            raw: (0b0100 << 4) | (w << 3) | (r << 2) | (x << 1) | b
        }
    }

    pub fn encode(&self) -> u8 {
        self.raw
    }

    pub fn decode(&self) -> (u8, u8, u8, u8) {
        (
            (self.raw >> 6) & 0b1,
            (self.raw >> 5) & 0b1,
            (self.raw >> 4) & 0b1,
            (self.raw >> 3) & 0b1
        )
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Register {
    RAX,
    RCX,
    RDX,
    RBX,
    RSP,
    RBP,
    RSI,
    RDI,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
    XMM0,
    XMM1,
    XMM2,
    XMM3,
    XMM4,
    XMM5,
    XMM6,
    XMM7,
    XMM8,
    XMM9,
    XMM10,
    XMM11,
    XMM12,
    XMM13,
    XMM14,
    XMM15,
}

// to_underlying for register
impl Register {
    fn encode(&self) -> u8 {
        let v = match self {
            Register::RAX | Register::XMM0  => 0,
            Register::RCX | Register::XMM1  => 1,
            Register::RDX | Register::XMM2  => 2,
            Register::RBX | Register::XMM3  => 3,
            Register::RSP | Register::XMM4  => 4,
            Register::RBP | Register::XMM5  => 5,
            Register::RSI | Register::XMM6  => 6,
            Register::RDI | Register::XMM7  => 7,
            Register::R8  | Register::XMM8  => 8,
            Register::R9  | Register::XMM9  => 9,
            Register::R10 | Register::XMM10 => 10,
            Register::R11 | Register::XMM11 => 11,
            Register::R12 | Register::XMM12 => 12,
            Register::R13 | Register::XMM13 => 13,
            Register::R14 | Register::XMM14 => 14,
            Register::R15 | Register::XMM15 => 15,
        };
        v & 0x7
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Immediate {
    Immediate8(u8),
    Immediate32(u32),
    Immediate64(u64),
}

impl Immediate {
    pub fn to_le_bytes(&self) -> Vec<u8> {
        match self {
            Immediate::Immediate8(i) => i.to_le_bytes().to_vec(),
            Immediate::Immediate32(i) => i.to_le_bytes().to_vec(),
            Immediate::Immediate64(i) => i.to_le_bytes().to_vec(),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Condition {
    Overflow,
    EqualTo,
    NotEqualTo,
    UnsignedGreaterThan,
    UnsignedGreaterThanOrEqualTo,
    UnsignedLessThan,
    UnsignedLessThanOrEqualTo,
    ParityEven,
    ParityOdd,
    SignedGreaterThan,
    SignedGreaterThanOrEqualTo,
    SignedLessThan,
    SignedLessThanOrEqualTo,
    Unordered,
    NotUnordered,
    Below,
    BelowOrEqual,
    Above,
    AboveOrEqual,
}

impl Condition {
    pub fn encode(&self) -> u8 {
        match self {
            Condition::Overflow => 0x0,
            Condition::EqualTo => 0x4,
            Condition::NotEqualTo => 0x5,
            Condition::UnsignedGreaterThan | Condition::Above => 0x7,
            Condition::UnsignedGreaterThanOrEqualTo | Condition::AboveOrEqual => 0x3,
            Condition::UnsignedLessThan | Condition::Below => 0x2,
            Condition::UnsignedLessThanOrEqualTo | Condition::BelowOrEqual => 0x6,
            Condition::ParityEven | Condition::Unordered => 0xA,
            Condition::ParityOdd | Condition::NotUnordered => 0xB,
            Condition::SignedGreaterThan => 0xF,
            Condition::SignedGreaterThanOrEqualTo => 0xD,
            Condition::SignedLessThan => 0xC,
            Condition::SignedLessThanOrEqualTo => 0xE,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Operand {
    // reg, is_float_register
    Register(Register, bool),
    // (base, offset)
    Memory(Register, u64),
    Immediate(Immediate),
}

impl Operand {
    pub fn is_register_or_memory(&self) -> bool {
        match self {
            Operand::Register(_, _) | Operand::Memory(_, _) => true,
            _ => false, 
        }
    }

    pub fn register_or_memory_base_to_underlying(&self) -> u8 {
        match self {
            Operand::Register(r, _) => r.encode(),
            Operand::Memory(r, _) => r.encode(),
            _ => unreachable!(),
        }
    }

    pub fn offset_or_immediate(&self) -> u64 {
        match self {
            Operand::Memory(_, offset) => *offset,
            Operand::Immediate(Immediate::Immediate8(i)) => *i as u64,
            Operand::Immediate(Immediate::Immediate32(i)) => *i as u64,
            Operand::Immediate(Immediate::Immediate64(i)) => *i,
            _ => unreachable!(),
        }
    }

    fn assert_immediate(&self) {
        assert!(match self {
            Operand::Immediate(_) => true,
            _ => false,
        });
    }

    pub fn fits_in_u8(&self) -> bool {
        self.assert_immediate();
        return self.offset_or_immediate() <= u8::MAX as u64;
    }

    pub fn fits_in_u32(&self) -> bool {
        self.assert_immediate();
        return self.offset_or_immediate() <= u32::MAX as u64;
    }

    pub fn fits_in_i8(&self) -> bool {
        self.assert_immediate();
        return self.offset_or_immediate() as i64 <= i8::MAX as i64;
    }

    pub fn fits_in_i32(&self) -> bool {
        self.assert_immediate();
        return self.offset_or_immediate() as i64 <= i32::MAX as i64;
    }
}

impl Label {
    pub fn new() -> Self {
        Self {
            offset: None,
            jump_slots: Vec::new(),
        }
    }

    pub fn add_jump(&mut self, assembler: &mut Assembler, offset: usize) {
        self.jump_slots.push(offset);
        if self.offset.is_some() {
            self.link_jump(assembler, offset);
        }
    }

    pub fn link(&mut self, assembler: &mut Assembler) {
        self.link_to(assembler, assembler.machine_code.len());
    }

    pub fn link_to(&mut self, assembler: &mut Assembler, link_offset: usize) {
        assert!(self.offset.is_none());

        self.offset = Some(link_offset);
        for offset in &self.jump_slots {
            self.link_jump(assembler, *offset);
        }
    }

    fn link_jump(&self, assembler: &mut Assembler, offset_in_instructions: usize) {
        let offset = self.offset.unwrap() - offset_in_instructions;
        let jump_slot = offset_in_instructions - 4;
        assembler.machine_code[jump_slot + 0] = ((offset >> 0) & 0xff) as u8;
        assembler.machine_code[jump_slot + 1] = ((offset >> 8) & 0xff) as u8;
        assembler.machine_code[jump_slot + 2] = ((offset >> 16) & 0xff) as u8;
        assembler.machine_code[jump_slot + 3] = ((offset >> 24) & 0xff) as u8;
    }
}

impl Assembler {
    pub fn new() -> Self {
        Self {
            machine_code: vec![],
        }
    }

    fn emit(&mut self, other: &[u8]) {
        self.machine_code.extend_from_slice(other);
    }

    fn emit8(&mut self, byte: u8) { self.machine_code.push(byte); }

    fn emit16(&mut self, word: u16) { self.emit(&word.to_le_bytes()); }

    fn emit32(&mut self, dword: u32) {
        self.emit(&dword.to_le_bytes());
    }

    fn emit64(&mut self, qword: u64) {
        self.emit(&qword.to_le_bytes());
    }

    fn emit_modrm(&mut self, raw: &mut ModRm, rm: &Operand) {
        assert!(match &rm {
            Operand::Immediate(_) => false,
            _ => true,
        });

        match &rm {
            Operand::Register(_, _) => {
                raw.set_mode(ModRmEnum::Reg.encode());
                self.emit8(raw.encode());
            },
            Operand::Memory(_, display) => {
                if *display == 0 {
                    raw.set_mode(ModRmEnum::Mem.encode());
                    self.emit8(raw.encode());
                } else if (*display as i64 >= -128) && (*display < 127) {
                    raw.set_mode(ModRmEnum::MemDisp8.encode());
                    self.emit8(raw.encode());
                    self.emit8((*display & 0xff) as u8);
                } else {
                    raw.set_mode(ModRmEnum::MemDisp32.encode());
                    self.emit8(raw.encode());
                    self.emit32(*display as u32);
                }
            }
            Operand::Immediate(_) => {
                unreachable!("emit_modrm: Operand::Immediate");
            }
        }
    }

    fn emit_modrm_rm(&mut self, dst: &Operand, src: &Operand) {
        assert!(match &src {
            Operand::Register(_, true) | Operand::Register(_, false) => true,
            _ => false
        });

        let mut raw = ModRm::new(
            dst.register_or_memory_base_to_underlying(), 
            src.register_or_memory_base_to_underlying(), 
            0
        );

        self.emit_modrm(&mut raw, src);
    }

    fn emit_modrm_mr(&mut self, dst: &Operand, src: &Operand) {
        assert!(match &src {
            Operand::Register(_, true) | Operand::Register(_, false) => true,
            _ => false
        });

        let mut raw = ModRm::new(dst.register_or_memory_base_to_underlying(), src.register_or_memory_base_to_underlying(), 0);

        self.emit_modrm(&mut raw, dst);
    }

    fn emit_modrm_slash(&mut self, slash: u8, rm: &Operand) {
        let mut raw = ModRm::new(rm.register_or_memory_base_to_underlying(), slash, 0);
        self.emit_modrm(&mut raw, rm);
    }

    fn emit_rex_for_rm(&mut self, dst: &Operand, src: &Operand, w: RexW) {
        assert!(src.is_register_or_memory() || match &src {
            Operand::Register(_, true) => true,
            _ => false
        });

        assert!(match &dst {
            Operand::Register(_, true) | Operand::Register(_, false) => true,
            _ => false
        });

        if w == RexW::No && dst.register_or_memory_base_to_underlying() < 8 && src.register_or_memory_base_to_underlying() < 8 { return; }

        let rex = Rex::new(
            (src.register_or_memory_base_to_underlying() >= 8) as u8,
            0, 
            (dst.register_or_memory_base_to_underlying() >= 8) as u8, 
            w.encode()
        );

        self.emit8(rex.encode()); 
    }

    fn emit_rex_for_mr(&mut self, dst: &Operand, src: &Operand, w: RexW) {
        assert!(dst.is_register_or_memory() || match &dst {
            Operand::Register(_, true) => true,
            _ => false
        });

        assert!(match &src {
            Operand::Register(_, true) | Operand::Register(_, false) => true,
            _ => false
        });

        if w == RexW::No && dst.register_or_memory_base_to_underlying() < 8 && src.register_or_memory_base_to_underlying() < 8 { return; }

        let rex = Rex::new(
            (dst.register_or_memory_base_to_underlying() >= 8) as u8,
            0, 
            (src.register_or_memory_base_to_underlying() >= 8) as u8, 
            w.encode()
        );

        self.emit8(rex.encode());
    }

    fn emit_rex_for_slash(&mut self, arg: &Operand, w: RexW) {
        assert!(arg.is_register_or_memory());

        if w == RexW::No && arg.register_or_memory_base_to_underlying() < 8 { return; }

        let rex = Rex::new(
            (arg.register_or_memory_base_to_underlying() >= 8) as u8, 
            0, 
            0, 
            w.encode()
        );
        self.emit8(rex.encode());
    }

    fn emit_rex_for_oi(&mut self, arg: &Operand, w: RexW) {
        self.emit_rex_for_slash(arg, w);
    }

    pub fn mov(&mut self, dst: Operand, src: Operand) {
        match (&dst, &src) {
            (_, Operand::Register(_, false)) if dst.is_register_or_memory() => {
                if dst.register_or_memory_base_to_underlying() == src.register_or_memory_base_to_underlying() { return; }
                self.emit_rex_for_mr(&dst, &src, RexW::Yes);
                self.emit8(0x89);
                self.emit_modrm_mr(&dst, &src);  
            },

            (Operand::Register(dst_reg, _), Operand::Immediate(_)) => {
                if src.offset_or_immediate() == 0 {
                    self.emit_rex_for_mr(&dst, &dst, RexW::No);
                    self.emit8(0x31);
                    self.emit_modrm_mr(&dst, &dst);
                    return;
                }

                if src.fits_in_u32() {
                    self.emit_rex_for_slash(&dst, RexW::No);
                    self.emit8(0xb8 | dst_reg.encode());
                    self.emit32(src.offset_or_immediate() as u32);
                    return;
                }

                self.emit_rex_for_oi(&dst, RexW::Yes);
                self.emit8(0xb8 | dst_reg.encode());
                self.emit64(src.offset_or_immediate());
            }
            _ => {
                // Handle other cases as needed
                dbg!(&dst, &src);
                todo!("Handle other cases in mov");
            }
        }
    }

    pub fn trap(&mut self) {
        self.machine_code.push(0xCC);
    }

    pub fn ret(&mut self, bytes: Option<u16>) {
        match bytes {
            Some(bytes) => {
                self.emit8(0xc2);
                self.emit16(bytes);
            },
            None => self.emit8(0xc3)
        }
    }

    pub fn leave(&mut self) {
        self.emit8(0xc9);
    }

    pub fn enter(&mut self) {
        self.push(Operand::Register(Register::RBP, false));
        self.mov(Operand::Register(Register::RBP, false), Operand::Register(Register::RSP, false));

        self.push_callee_saved_registers();
    }

    pub fn exit(&mut self) {
        self.pop_callee_saved_registers();
        
        // leave
        self.leave();
        
        // ret
        self.ret(None);
    }

    pub fn push(&mut self, op: Operand) {
        match &op {
            Operand::Register(reg, _) => {
                self.emit_rex_for_oi(&op, RexW::No);
                self.emit8(0x50 | reg.encode());
            }
            Operand::Immediate(_) if op.fits_in_i8() => {
                    self.emit8(0x6a);
                    self.emit8(op.offset_or_immediate() as u8);
            }
            Operand::Immediate(_) if op.fits_in_i32() => {
                self.emit8(0x68);
                self.emit32(op.offset_or_immediate() as u32);
            }
            _ => {
                unreachable!("push: Invalid operand");
            }
        }
    }

    pub fn pop(&mut self, op: Operand) {
        match &op {
            Operand::Register(reg, _) => {
                self.emit_rex_for_oi(&op, RexW::No);
                self.emit8(0x58 | reg.encode());
            }
            _ => {
                unreachable!("pop: Invalid operand");
            }
        }
    }

    pub fn push_callee_saved_registers(&mut self) {
        self.push(Operand::Register(Register::RBX, false));
        self.push(Operand::Register(Register::RBP, false));
        self.push(Operand::Register(Register::R12, false));
        self.push(Operand::Register(Register::R13, false));
        self.push(Operand::Register(Register::R14, false));
        self.push(Operand::Register(Register::R15, false));
    }

    pub fn pop_callee_saved_registers(&mut self) {
        self.pop(Operand::Register(Register::R15, false));
        self.pop(Operand::Register(Register::R14, false));
        self.pop(Operand::Register(Register::R13, false));
        self.pop(Operand::Register(Register::R12, false));
        self.pop(Operand::Register(Register::RBP, false));
        self.pop(Operand::Register(Register::RBX, false));
    }

    pub fn sub(&mut self, dst: Operand, src: Operand) {
        match (&dst, &src) {
            (_, Operand::Register(_, false)) if dst.is_register_or_memory() => {
                self.emit_rex_for_mr(&dst, &src, RexW::Yes);
                self.emit8(0x29);
                self.emit_modrm_mr(&dst, &src);
            }
            (_, Operand::Immediate(_)) if dst.is_register_or_memory() && src.fits_in_i8() => {
                self.emit_rex_for_slash(&dst, RexW::Yes);
                self.emit8(0x83);
                self.emit_modrm_slash(5, &dst);
                self.emit8(src.offset_or_immediate() as u8);
            }
            (_, Operand::Immediate(_)) if dst.is_register_or_memory() && src.fits_in_i32() => {
                self.emit_rex_for_slash(&dst, RexW::Yes);
                self.emit8(0x81);
                self.emit_modrm_slash(5, &dst);
                self.emit32(src.offset_or_immediate() as u32);
            }
            (Operand::Register(_, true), Operand::Register(_, true)) => {
                self.emit8(0xf2);
                self.emit8(0x0f);
                self.emit8(0x5c);
                self.emit_modrm_rm(&dst, &src);
            }
            _ => {
                unreachable!("sub: Invalid operands");
            }
        }
    }

    pub fn add(&mut self, dst: Operand, src: Operand) {
        match (&dst, &src) {
            (_, Operand::Register(_, false)) if dst.is_register_or_memory() => {
                self.emit_rex_for_mr(&dst, &src, RexW::Yes);
                self.emit8(0x01);
                self.emit_modrm_mr(&dst, &src);
            },
            (_, Operand::Immediate(_)) if dst.is_register_or_memory() && src.fits_in_i8() => {
                self.emit_rex_for_slash(&dst, RexW::Yes);
                self.emit8(0x83);
                self.emit_modrm_slash(0, &dst);
                self.emit8(src.offset_or_immediate() as u8);
            },
            (_, Operand::Immediate(_)) if dst.is_register_or_memory() && src.fits_in_i32() => {
                self.emit_rex_for_slash(&dst, RexW::Yes);
                self.emit8(0x81);
                self.emit_modrm_slash(0, &dst);
                self.emit32(src.offset_or_immediate() as u32);
            },
            (Operand::Register(_, true), Operand::Register(_, true)) => {
                self.emit8(0xf2);
                self.emit8(0x0f);
                self.emit8(0x58);
                self.emit_modrm_rm(&dst, &src);
            }
            _ => {
                unreachable!("add: Invalid operands");
            }
        }
    }

    pub fn inc32(&mut self, op: &Operand, label: Option<&mut Label>) {
        match &op {
            _ if op.is_register_or_memory() => {
                self.emit_rex_for_slash(op, RexW::No);
                self.emit8(0xff);
                self.emit_modrm_slash(0, op);
            },
            _ => {
                unreachable!("inc32: Invalid operands");
            }
        }

        if let Some(label) = label {
            self.jump_if_label(Condition::Overflow, label);
        }
    }

    pub fn dec32(&mut self, op: &Operand, label: Option<&mut Label>) {
        match &op {
            _ if op.is_register_or_memory() => {
                self.emit_rex_for_slash(op, RexW::No);
                self.emit8(0xff);
                self.emit_modrm_slash(1, op);
            },
            _ => {
                unreachable!("dec32: Invalid operands");
            }
        }

        if let Some(label) = label {
            self.jump_if_label(Condition::Overflow, label);
        }
    }

    pub fn jump(&mut self) -> Label {
        // jmp target (RIP-relative 32bit)
        self.emit8(0xe9);
        self.emit32(0xdeadbeef);
        let mut label = Label::new();
        label.add_jump(self, self.machine_code.len());
        label
    }

    pub fn jump_label(&mut self, label: &mut Label) {
        self.emit8(0xe9);
        self.emit32(0xdeadbeef);
        label.add_jump(self, self.machine_code.len());
    }

    pub fn jump_op(&mut self, op: &Operand) {
        self.emit_rex_for_slash(op, RexW::No);
        self.emit8(0xff);
        self.emit_modrm_slash(4, op);
    }

    pub fn jump_if_label(&mut self, condition: Condition, label: &mut Label) {
        self.emit8(0x0f);
        self.emit8(0x80 | condition.encode());
        self.emit32(0xdeadbeef);
        label.add_jump(self, self.machine_code.len());
    }

    pub fn jump_if_cmp(&mut self, lhs: &Operand, condition: Condition, rhs: &Operand, label: &mut Label) {
        self.cmp(lhs, rhs);
        self.jump_if_label(condition, label);
    }

    pub fn set_if(&mut self, condition: Condition, dst: &Operand) {
        self.emit_rex_for_slash(dst, RexW::No);
        self.emit8(0x0f);
        self.emit8(0x90 | condition.encode());
        self.emit_modrm_slash(0, dst);
    }

    pub fn mov_if(&mut self, condition: Condition, dst: &Operand, src: &Operand) {
        assert!(match (&dst, &src) {
            (Operand::Register(_, _), Operand::Register(_, _)) => true,
            _ => false
        });

        self.emit_rex_for_mr(dst, src, RexW::Yes);
        self.emit8(0x0f);
        self.emit8(0x40 | condition.encode());
        self.emit_modrm_rm(dst, src);
    }

    pub fn test(&mut self, lhs: &Operand, rhs: &Operand) {
        unimplemented!("test");
    }

    pub fn cmp(&mut self, lhs: &Operand, rhs: &Operand) {
        match (&lhs, &rhs) {
            (Operand::Register(_, _), Operand::Immediate(_)) if rhs.offset_or_immediate() == 0 => {
                self.test(&lhs, &rhs);
            },
            (_, Operand::Register(_, _)) if lhs.is_register_or_memory() => {
                self.emit_rex_for_mr(lhs, rhs, RexW::Yes);
                self.emit8(0x39);
                self.emit_modrm_mr(lhs, rhs);
            },
            (_, Operand::Immediate(_)) if lhs.is_register_or_memory() && rhs.fits_in_i8() => {
                self.emit_rex_for_slash(lhs, RexW::Yes);
                self.emit8(0x83);
                self.emit_modrm_slash(7, lhs);
                self.emit8(rhs.offset_or_immediate() as u8);
            },
            (_, Operand::Immediate(_)) if lhs.is_register_or_memory() && rhs.fits_in_i32() => {
                self.emit_rex_for_slash(lhs, RexW::Yes);
                self.emit8(0x81);
                self.emit_modrm_slash(7, lhs);
                self.emit32(rhs.offset_or_immediate() as u32);
            },
            (Operand::Register(_, true), Operand::Register(_, true)) |
            (Operand::Register(_, true), Operand::Memory(_, _)) => {
                // ucomisd lhs, rhs
                self.emit8(0x66);
                self.emit_rex_for_rm(lhs, rhs, RexW::No);
                self.emit8(0x0f);
                self.emit8(0x2e);
                self.emit_modrm_rm(lhs, rhs);
            }
            _ => {
                unreachable!("cmp: Invalid operands");
            }
        }
    }

    pub fn call_rax(&mut self) {
        self.emit8(0xff);
        self.emit_modrm_slash(2, &Operand::Register(Register::RAX, false));
    }

}