use winapi::ctypes::c_void;
use std::ptr;
use winapi::um::memoryapi::{VirtualAlloc, VirtualFree};
use winapi::um::winnt::{MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_EXECUTE_READWRITE};

use crate::kod::parser::node::Node;
use super::assembler::{Assembler, Immediate, Operand, Register, disassemble_machine_code};

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


impl JitCompiler {
    pub fn new() -> Self {
        Self {
            assembler: Assembler::new(),
        }
    }

    pub fn compile(&mut self, _ast: Box<dyn Node>) {

        println!("{:x?}", self.assembler.machine_code);
        disassemble_machine_code(&self.assembler.machine_code);

    }

    pub fn run(&mut self) { 
        let jit_function = JitFunction::new(&self.assembler.machine_code);
        println!("JIT Result: {:x}", jit_function.run());
    }
}