use std::ops::Deref;
use std::vec;
use std::ptr;
use winapi::um::memoryapi::{VirtualAlloc, VirtualFree};
use winapi::um::winnt::{MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_EXECUTE_READWRITE};

use crate::kod::parser::node::Node;

use capstone::prelude::*;

fn disassemble_machine_code(machine_code: &[u8]) {
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
    machine_code: Vec<u8>,
}

pub struct JitCompiler {
    pub assembler: Assembler,
}

pub struct ModRm {
    pub raw: u8,
}

pub enum ModRmEnum {
    Mem,
    MemDisp8,
    MemDisp32,
    Reg,
}

impl ModRmEnum {
    pub fn encode(&self) -> u8 {
        match self {
            ModRmEnum::Mem => 0b00,
            ModRmEnum::MemDisp8 => 0b01,
            ModRmEnum::MemDisp32 => 0b10,
            ModRmEnum::Reg => 0b11,
        }
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
pub enum RexW {
    Yes,
    No,
}

impl RexW {
    pub fn encode(&self) -> u8 {
        match self {
            RexW::Yes => 1,
            RexW::No => 0,
        }
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
        match self {
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
        }
    }
}

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

pub enum Operand {
    Register(Register, bool), // reg, is_f
    Memory(Register, u64), // (base, offset)
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

    pub fn does_fit_in_32(&self) -> bool {
        return self.offset_or_immediate() <= u32::MAX as u64;
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

    fn emit8(&mut self, byte: u8) {
        self.machine_code.push(byte);
    }

    fn emit32(&mut self, word: u32) {
        self.machine_code.extend_from_slice(&word.to_le_bytes());
    }

    fn emit64(&mut self, dword: u64) {
        self.machine_code.extend_from_slice(&dword.to_le_bytes());
    }

    fn emit_modrm(&mut self, raw: &mut ModRm, rm: Operand) {
        assert!(match &rm {
            Operand::Immediate(_) => false,
            _ => true,
        });

        match &rm {
            Operand::Register(r, _) => {
                raw.set_mode(ModRmEnum::Reg.encode());
                self.emit8(raw.encode());
            },
            Operand::Memory(r, disp) => {
                if *disp == 0 {
                    raw.set_mode(ModRmEnum::Mem.encode());
                    self.emit8(raw.encode());
                } else if (*disp as i64 >= -128) && (*disp < 127) {
                    raw.set_mode(ModRmEnum::MemDisp8.encode());
                    self.emit8(raw.encode());
                    self.emit8((*disp & 0xff) as u8);
                } else {
                    raw.set_mode(ModRmEnum::MemDisp32.encode());
                    self.emit8(raw.encode());
                    self.emit32(*disp as u32);
                }
            },
            Operand::Immediate(_) => {
                unreachable!("emit_modrm: Operand::Immediate");
            }
        }
    }

    fn emit_modrm_mr(&mut self, dst: &Operand, src: &Operand) {
        unimplemented!("emit_modrm_mr");
    }

    fn emit_modrm_slash(&mut self, slash: u8, rm: Operand) {
        let mut raw = ModRm::new(rm.register_or_memory_base_to_underlying(), slash, 0);
        self.emit_modrm(&mut raw, rm);
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

    fn emit_rex_for_OI(&mut self, arg: &Operand, w: RexW) {
        self.emit_rex_for_slash(arg, w);
    }

    pub fn mov(&mut self, dst: Operand, src: Operand) {
        match (&dst, &src) {
            (_, Operand::Register(src_reg, true)) if dst.is_register_or_memory() => {
                if dst.register_or_memory_base_to_underlying() == src.register_or_memory_base_to_underlying() { return; }
                self.emit_rex_for_mr(&dst, &src, RexW::Yes);
                self.emit8(0x89);
                self.emit_modrm_mr(&dst, &src);  
            },

            (Operand::Register(dst_reg, _), Operand::Immediate(_)) => {
                if src.does_fit_in_32() {
                    self.emit_rex_for_slash(&dst, RexW::No);
                    self.emit8(0xb8 | dst_reg.encode());
                    self.emit32(src.offset_or_immediate() as u32);
                    return;
                }

                self.emit_rex_for_OI(&dst, RexW::Yes);
                self.emit8(0xb8 | dst_reg.encode());
                self.emit64(src.offset_or_immediate());
            }
            _ => {
                // Handle other cases as needed
                todo!("Handle other cases in mov");
            }
        }
    }

    pub fn ret(&mut self) {
        self.machine_code.push(0xC3);
    }
}

fn test() -> u64 {
    println!("Hello from test!");
    return 0x69420;
}

impl JitCompiler {
    pub fn new() -> Self {
        Self {
            assembler: Assembler::new(),
        }
    }

    pub fn compile(&mut self, ast: Box<dyn Node>) {
        self.assembler.mov(Operand::Register(Register::RAX, false), Operand::Immediate(Immediate::Immediate64(test as u64)));
        self.assembler.emit8(0xff);
        self.assembler.emit_modrm_slash(2, Operand::Register(Register::RAX, false));
        self.assembler.ret();

        println!("{:x?}", self.assembler.machine_code);
        disassemble_machine_code(&self.assembler.machine_code);

    }

    pub fn run(&mut self) { 
        // Allocate virtual memory
        let code_size = self.assembler.machine_code.len();
        let code_ptr = unsafe {
            VirtualAlloc(
                ptr::null_mut(),
                code_size,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_EXECUTE_READWRITE,
            )
        };

        if code_ptr.is_null() {
            // Handle error
            panic!("Failed to allocate virtual memory");
        }

        // Copy machine code to allocated memory
        unsafe {
            ptr::copy_nonoverlapping(
                self.assembler.machine_code.as_ptr(),
                code_ptr as *mut u8,
                code_size,
            );
        }

        // Define a function pointer with the appropriate signature
        type JitFunction = fn() -> i64;
        let jit_function: JitFunction = unsafe { std::mem::transmute(code_ptr) };

        // Call the generated code
        let result = jit_function();

        // Free the allocated virtual memory
        unsafe {
            VirtualFree(code_ptr, 0, MEM_RELEASE);
        }

        println!("JIT Result: {:x}", result);
    }
}