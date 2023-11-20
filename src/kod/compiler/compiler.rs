use winapi::ctypes::c_void;
use std::ptr;
use winapi::um::memoryapi::{VirtualAlloc, VirtualFree};
use winapi::um::winnt::{MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_EXECUTE_READWRITE};

use crate::kod::parser::node::Node;
use super::assembler::{Assembler, Condition, Immediate, Label, Operand, Register, disassemble_machine_code};

pub struct JitCompiler {
    pub assembler: Assembler,
}

pub struct JitFunction {
    function: fn() -> i64,
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
        type JitFunction = fn() -> i64;
        let function: JitFunction = unsafe { std::mem::transmute(code_ptr) };
        Self {
            function,
            code_ptr,
        }
    }

    pub fn run(&self) -> i64 {
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

#[derive(Debug)]
pub struct FunctionObject {
    function: u64,
    name: String,
}

#[derive(Debug, Copy, Clone)]
pub struct Object(u64);


#[derive(Debug, Copy, Clone)]
pub enum ObjectTag {
    Int,
    Float,
    Pointer,
}

impl ObjectTag {
    pub fn from(value: u16) -> Self {
        match value {
            0 => Self::Int,
            1 => Self::Float,
            2 => Self::Pointer,
            _ => panic!("Invalid object tag"),
        }
    }
}

impl Object {
    pub fn new(value: u64, object_tag: ObjectTag) -> Self {
        Self(((object_tag as u64) << 48) as u64 | (value & 0x0000_FFFF_FFFF_FFFF) as u64)
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

fn test(x: Object) -> u64 {
    println!("Hello from test! {x:?}, {:?}", x.tag());

    match x.tag() {
        ObjectTag::Int => (),
        ObjectTag::Float => (),
        ObjectTag::Pointer => {
            let func: fn(Object) -> u64 = unsafe {std::mem::transmute(x.value())};
            return func(Object::new(69, ObjectTag::Int));
        },
    };

    return 0x69420;
}

impl JitCompiler {
    pub fn new() -> Self {
        Self {
            assembler: Assembler::new(),
        }
    }

    pub fn compile(&mut self, ast: Box<dyn Node>) {

        let a = Object::new(69, ObjectTag::Int);
        let b = FunctionObject {
            function: test as u64,
            name: "test".to_string(),
        };

        let b_o = Object::new(b.function, ObjectTag::Pointer);
        
        self.assembler.enter();
        self.assembler.mov(Operand::Register(Register::RCX, false), Operand::Immediate(Immediate::Immediate64(b_o.encode())));
        self.assembler.mov(Operand::Register(Register::RAX, false), Operand::Immediate(Immediate::Immediate64(b_o.value())));
        self.assembler.call_rax();
        self.assembler.exit();

        println!("{:x?}", self.assembler.machine_code);
        disassemble_machine_code(&self.assembler.machine_code);



    }

    pub fn run(&mut self) { 
        let jit_function = JitFunction::new(&self.assembler.machine_code);
        println!("JIT Result: {:x}", jit_function.run());
    }
}