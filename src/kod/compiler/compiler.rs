use winapi::ctypes::c_void;
use std::ptr;
use winapi::um::memoryapi::{VirtualAlloc, VirtualFree};
use winapi::um::winnt::{MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_EXECUTE_READWRITE};

use crate::kod::parser::node::Node;
use super::assembler::{Assembler, Immediate, Operand, Register, disassemble_machine_code, Condition, Label};


#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Object(u64);


#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ObjectTag {
    Int,
    Null,
    Float,
    Pointer,
    Code,
    NativeFunc,
}

impl ObjectTag {
    pub fn from(value: u16) -> Self {
        match value {
            0 => Self::Int,
            1 => Self::Null,
            2 => Self::Float,
            3 => Self::Pointer,
            4 => Self::Code,
            5 => Self::NativeFunc,
            _ => panic!("Invalid object tag"),
        }
    }
}

impl Object {
    pub fn new(value: u64, object_tag: ObjectTag) -> Self {
        Self(((object_tag as u64) << 48) as u64 | (value & 0x0000_FFFF_FFFF_FFFF) as u64)
    }

    pub fn from(value: u64) -> Self {
        Self((value & 0xFFFF_0000_0000_0000) | (value & 0x0000_FFFF_FFFF_FFFF))
    }

    pub fn encode(&self) -> u64 {
        self.0
    }

    pub fn value(&self) -> u64 {
        self.0 & 0x0000_FFFF_FFFF_FFFF
    }

    pub fn tag(&self) -> ObjectTag {
        ObjectTag::from(((self.0 >> 48) & 0xFFFF) as u16)
    }
}

#[derive(Debug, Clone)]
pub enum InstructionEnum {
    Mov(Operand, Operand),
    Push(Operand),
    Pop(Operand),
    JumpBytecode(u32), // bytecode offset to jump to
    JumpBytecodeIfCmp(Operand, Condition, Operand, u32), // lhs, cond, rhs, jump_to in bytecode
    Exit,
    Call(u64),
    Jump,
    Shr(Register, u8),
    IntFastSlowPathBinary(u64, u64),
}

#[derive(Debug, Clone)]
pub struct Instruction {
    bytecode_offset: u32,
    offset: usize,
    ins: InstructionEnum,
    label: Label,
}

impl Instruction {
    pub fn new(bytecode_offset: u32, ins: InstructionEnum) -> Self {
        Self {
            bytecode_offset,
            offset: 0,
            ins,
            label: Label::new(),
        }
    }
}

pub struct JitCompiler {
    pub assembler: Assembler,
    instructions: Vec<Instruction>,
}

pub struct JitFunction {
    function: fn() -> u64,
    code_ptr: *mut c_void,
}

impl JitFunction {
    pub fn new(machine_code: &[u8]) -> Self {
        let code_size = machine_code.len();
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
                machine_code.as_ptr(),
                code_ptr as *mut u8,
                code_size,
            );
        }

        // Define a function pointer with the appropriate signature
        type JitFunction = fn() -> u64;
        let function: JitFunction = unsafe { std::mem::transmute(code_ptr) };
        Self {
            function,
            code_ptr,
        }
    }

    pub fn run(&self) -> u64 {
        (self.function)()
    }

}

impl Drop for JitFunction {
    fn drop(&mut self) {
        unsafe {
            VirtualFree(self.code_ptr, 0, MEM_RELEASE);
        }
    }
}


impl JitCompiler {
    pub fn new() -> Self {
        Self {
            assembler: Assembler::new(),
            instructions: Vec::new(),
        }
    }

    pub fn compile(&mut self) {
        self.optimize();

        self.assembler.enter();

        for ins_index in 0..self.instructions.len() {
            self.instructions[ins_index].offset = self.assembler.machine_code.len();
            let ins = &self.instructions[ins_index].ins;
            match ins {
                InstructionEnum::Jump => {
                    self.instructions[ins_index].label = self.assembler.jump();
                },
                InstructionEnum::JumpBytecode(_) => {
                    self.instructions[ins_index].label = self.assembler.jump();
                },
                InstructionEnum::JumpBytecodeIfCmp(lhs, condition, rhs, _) => {
                    let mut label = Label::new();
                    self.assembler.jump_if_cmp(&lhs, *condition, &rhs, &mut label);
                    self.instructions[ins_index].label = label;
                }
                InstructionEnum::Pop(op) => {
                    self.assembler.pop(*op);
                },
                InstructionEnum::Push(op) => {
                    self.assembler.push(*op);
                },
                InstructionEnum::Mov(dst, src) => {
                    self.assembler.mov(*dst, *src);
                },
                InstructionEnum::Exit => {
                    self.assembler.exit();
                },
                InstructionEnum::Call(callee) => {
                    self.assembler.mov(Operand::Register(Register::RAX, false), Operand::Immediate(Immediate::Immediate64(*callee)));
                    self.assembler.call_rax();
                },
                InstructionEnum::Shr(register, amount) => {
                    self.assembler.shr(&Operand::Register(*register, false), &Operand::Immediate(Immediate::Immediate8(*amount)));
                },
                InstructionEnum::IntFastSlowPathBinary(vm, op) => {
                    let mut end = Label::new();            
                    let mut slow_path = Label::new();     

                    self.assembler.jump_if_cmp(
                        &Operand::Register(Register::RAX, false), 
                        Condition::NotEqualTo, 
                        &Operand::Immediate(Immediate::Immediate64(ObjectTag::Int as u64)), 
                        &mut slow_path
                    );
                    
                    self.assembler.jump_if_cmp(
                        &Operand::Register(Register::RBX, false), 
                        Condition::NotEqualTo, 
                        &Operand::Immediate(Immediate::Immediate64(ObjectTag::Int as u64)), 
                        &mut slow_path
                    );

                    // add them!!!
                    self.assembler.add(Operand::Register(Register::R8, false), Operand::Register(Register::RDX, false));
                    self.assembler.mov(Operand::Register(Register::RAX, false), Operand::Register(Register::R8, false));

                    self.assembler.jump_label(&mut end);
                    slow_path.link(&mut self.assembler);

                    self.assembler.mov(Operand::Register(Register::RCX, false), Operand::Immediate(Immediate::Immediate64(*vm)));
                    self.assembler.mov(Operand::Register(Register::RAX, false), Operand::Immediate(Immediate::Immediate64(*op)));
                    self.assembler.call_rax();

                    end.link(&mut self.assembler);
                }
                _ => unimplemented!("{:#?}", ins),
            };
        }

        for ins_index in 0..self.instructions.len() {
            let ins = &self.instructions[ins_index].ins;
            match ins {
                InstructionEnum::Jump => {
                    // find the instruction
                    unimplemented!("{} {:#?}", ins_index, ins);
                    // self.instructions[ins_index].label.link_to(&mut self.assembler, ins_offset);
                },
                InstructionEnum::JumpBytecode(jump_to) => {
                    let ins_offset = self.instructions.iter().find(|ins| { ins.bytecode_offset == *jump_to })
                        .expect("Failed to find jump offset")
                        .offset;

                    self.instructions[ins_index].label.link_to(&mut self.assembler, ins_offset);
                },
                InstructionEnum::JumpBytecodeIfCmp(lhs, condition, rhs, jump_to) => {
                    let ins_offset = self.instructions.iter().find(|ins| { ins.bytecode_offset == *jump_to })
                        .expect("Failed to find jump offset")
                        .offset;

                    self.instructions[ins_index].label.link_to(&mut self.assembler, ins_offset);
                },
                _ => {},
            }
        }

        println!("{:?}", self.instructions);
        println!("{:x?}", self.assembler.machine_code);
        disassemble_machine_code(&self.assembler.machine_code);
    }

    fn optimize(&mut self) {
        let mut new_instructions: Vec<Instruction> = vec![];
        let mut i: usize = 0;

        while i < self.instructions.len() {
            let ins = &self.instructions[i].ins;
            match ins {
                InstructionEnum::Mov(dst, src) => {
                    // mov x, src
                    // push x
                    // pop dst
                    // ------------
                    // mov dst, src
                    if let Some(ins2) = self.instructions.get(i + 1) {
                        match ins2.ins {
                            InstructionEnum::Push(should_be_dst) => {
                                if *dst == should_be_dst {
                                    if let Some(ins3) = self.instructions.get(i + 2) {
                                        match ins3.ins {
                                            InstructionEnum::Pop(dst) => {
                                                new_instructions.push(Instruction::new(self.instructions[i].bytecode_offset, InstructionEnum::Mov(dst, *src)));
                                                i += 3;
                                                continue;
                                            }
                                            _ => {
                                                // idk
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {
                                // idk
                            }
                        }
                    }
                },
                InstructionEnum::Push(op) => {
                    // push op
                    // pop op
                    // ------ nothing
                    if let Some(ins2) = self.instructions.get(i + 1) {
                        match ins2.ins {
                            InstructionEnum::Pop(should_be_op) => {
                                if *op == should_be_op {
                                    i += 2;
                                    continue;
                                }
                            }
                            _ => {
                                // push op1, ... (op1 is not changing), pop op2 -> mov op2, op1
                            }
                        }
                    }
                },
                _ => {
                },
            }
            new_instructions.push(self.instructions[i].clone());
            i += 1;
        }

        self.instructions = new_instructions;
    }

    pub fn compile_jump_bytecode(&mut self, bytecode_offset: u32, jump_to: u32) {
        self.instructions.push(Instruction::new(bytecode_offset, InstructionEnum::JumpBytecode(jump_to)));
    }

    pub fn compile_pop_jump_if_false_bytecode(&mut self, bytecode_offset: u32, jump_to: u32) {
        self.instructions.push(Instruction::new(bytecode_offset, InstructionEnum::Pop(Operand::Register(Register::RAX, false))));
        self.instructions.push(Instruction::new(bytecode_offset, InstructionEnum::JumpBytecodeIfCmp(
            Operand::Register(Register::RAX, false),
            Condition::EqualTo,
            Operand::Immediate(Immediate::Immediate8(0)),
            jump_to))
        );
    }

    pub fn compile_jump(&mut self, bytecode_offset: u32) {
        unimplemented!("compile_jump")
    }
    
    pub fn compile_jump_if_not_equal(&mut self, bytecode_offset: u32) {
        unimplemented!("compile_pop_jump_if_not_equal")
    }

    pub fn compile_pop(&mut self, bytecode_offset: u32, register: Register) {
        self.instructions.push(Instruction::new(bytecode_offset, InstructionEnum::Pop(Operand::Register(register, false))));
    }

    pub fn compile_push(&mut self, bytecode_offset: u32, op: Operand) {
        self.instructions.push(Instruction::new(bytecode_offset, InstructionEnum::Push(op)));
    }

    pub fn compile_push_immediate(&mut self, bytecode_offset: u32, immediate: Immediate) {
        self.instructions.push(Instruction::new(bytecode_offset,
                                                InstructionEnum::Mov(
                                                    Operand::Register(Register::RAX, false),
                                                    Operand::Immediate(immediate)
                                                )));
        self.instructions.push(Instruction::new(bytecode_offset, InstructionEnum::Push(Operand::Register(Register::RAX, false))));
    }

    pub fn compile_return(&mut self, bytecode_offset: u32) {
        self.compile_pop(bytecode_offset, Register::RAX);
        self.instructions.push(Instruction::new(bytecode_offset, InstructionEnum::Exit));
    }

    pub fn compile_mov(&mut self, bytecode_offset: u32, dst: Operand, src: Operand) {
        self.instructions.push(Instruction::new(bytecode_offset, InstructionEnum::Mov(dst, src)));
    }

    pub fn compile_call(&mut self, bytecode_offset: u32, callee: u64, args: Vec<Operand>) {
        let integer_regs = vec![
            Operand::Register(Register::RCX, false),
            Operand::Register(Register::RDX, false),
            Operand::Register(Register::R8, false),
            Operand::Register(Register::R9, false)
        ];

        let float_regs = vec![
            Operand::Register(Register::XMM0, true),
            Operand::Register(Register::XMM1, true),
            Operand::Register(Register::XMM2, true),
            Operand::Register(Register::XMM3, true)
        ];

        assert!(args.len() <= integer_regs.len());

        for i in 0..args.len() {
            self.compile_mov(bytecode_offset, integer_regs[i], args[i]);
        }

        self.instructions.push(Instruction::new(bytecode_offset, InstructionEnum::Call(callee)));
    }

    pub fn compile_shr(&mut self, bytecode_offset: u32, register: Register, amount: u8) {
        self.instructions.push(Instruction::new(bytecode_offset, InstructionEnum::Shr(register, amount)));
    }

    pub fn compile_binary_add(&mut self, bytecode_offset: u32, vm: u64, op: u64) {
        self.compile_pop(bytecode_offset, Register::R8); // first right
        self.compile_pop(bytecode_offset, Register::RDX); // then left

        // rax = lhs >> 48
        self.compile_mov(bytecode_offset, Operand::Register(Register::RAX, false), Operand::Register(Register::RDX, false));
        self.compile_shr(bytecode_offset, Register::RAX, 48);
        
        // rbx = rhs >> 48
        self.compile_mov(bytecode_offset, Operand::Register(Register::RBX, false), Operand::Register(Register::R8, false));
        self.compile_shr(bytecode_offset, Register::RBX, 48);

        self.instructions.push(Instruction::new(bytecode_offset, InstructionEnum::IntFastSlowPathBinary(vm, op)));
    }

    pub fn run(&mut self) -> u64 { 
        let jit_function = JitFunction::new(&self.assembler.machine_code);
        jit_function.run()
    }
}