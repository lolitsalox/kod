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

#[repr(u8)]
pub enum XMMRegister {
    XMM0 = 0,
    XMM1 = 1,
    XMM2 = 2,
    XMM3 = 3,
    XMM4 = 4,
    XMM5 = 5,
    XMM6 = 6,
    XMM7 = 7,
    XMM8 = 8,
    XMM9 = 9,
    XMM10 = 10,
    XMM11 = 11,
    XMM12 = 12,
    XMM13 = 13,
    XMM14 = 14,
    XMM15 = 15,
}

#[repr(u8)]
pub enum Register {
    RAX = 0,
    RCX = 1,
    RDX = 2,
    RBX = 3,
    RSP = 4,
    RBP = 5,
    RSI = 6,
    RDI = 7,
    R8 = 8,
    R9 = 9,
    R10 = 10,
    R11 = 11,
    R12 = 12,
    R13 = 13,
    R14 = 14,
    R15 = 15,
    XMM(XMMRegister),
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
    Register(Register),
    Memory(Register, u64), // (base, offset)
    Immediate(Immediate),
}

impl Assembler {
    pub fn new() -> Self {
        Self {
            machine_code: vec![],
        }
    }

    pub fn emit8(&mut self, byte: u8) {
        self.machine_code.push(byte);
    }

    pub fn emit32(&mut self, word: u32) {
        self.machine_code.extend_from_slice(&word.to_le_bytes());
    }

    pub fn emit64(&mut self, dword: u64) {
        self.machine_code.extend_from_slice(&dword.to_le_bytes());
    }

    pub fn mov(&mut self, dst: Operand, src: Operand) {
        match (dst, src) {
            (Operand::Register(dst_reg), Operand::Immediate(imm)) => {
                // Generate mov instruction for moving immediate to register
                self.machine_code.push(0x48); // REX prefix for 64-bit operands
                match imm {
                    Immediate::Immediate8(_) => self.machine_code.push(0xB0), // Opcode for mov with immediate operand
                    Immediate::Immediate32(_) => self.machine_code.push(0xC0), // Opcode for mov with immediate operand
                    Immediate::Immediate64(_) => self.machine_code.push(0xC7), // Opcode for mov with immediate operand
                }
                match dst_reg {
                    Register::RAX => self.machine_code.push(0xC0), // ModR/M byte for RAX
                    Register::RBX => self.machine_code.push(0xC3), // ModR/M byte for RBX
                }
                
                self.machine_code.extend_from_slice(&imm.to_le_bytes());
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

impl JitCompiler {
    pub fn new() -> Self {
        Self {
            assembler: Assembler::new(),
        }
    }

    pub fn compile(&mut self, ast: Box<dyn Node>) {
        self.assembler.mov(Operand::Register(Register::RAX), Operand::Immediate(Immediate::Immediate64(69)));
        self.assembler.ret();

        // disassemble_machine_code(&vec![0x48, 0xC7, 0xC0, 0x69, 0x00, 0x00, 0x00]);


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

        println!("JIT Result: {}", result);
    }
}