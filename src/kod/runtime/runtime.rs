use crate::kod::{compiler::{bytekod::{Code, Module, Opcode, Constant}, compiler::JitCompiler, assembler::{Operand, Register, Immediate, Label, Condition}}, parser::node::{Node, IntNode, NodeEnum}, lexer::token::TokenType};

use std::{cell::RefCell, collections::VecDeque};
use std::collections::{HashSet, LinkedList};
use std::hash::{Hash, Hasher};
use std::rc::{Rc, Weak};
use crate::kod::compiler::assembler::Immediate::{Immediate32, Immediate64, Immediate8};
use crate::kod::compiler::compiler::{Object, ObjectTag};

#[derive(Debug)]
enum VMValue {
    Null,
    String(String),
    Code(Code),
}

#[derive(Debug)]
struct VMObject {
    value: VMValue,
    marked: bool,
}

#[derive(Debug, Clone)]
struct WeakObjectRef(Weak<RefCell<VMObject>>);

impl Hash for WeakObjectRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.as_ptr().hash(state);
    }
}

impl PartialEq for WeakObjectRef {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ptr() == other.0.as_ptr()
    }
}

impl Eq for WeakObjectRef {}

struct GarbageCollector {
    objects: LinkedList<Rc<RefCell<VMObject>>>,
    roots: HashSet<WeakObjectRef>,
}

impl GarbageCollector {
    fn new() -> GarbageCollector {
        GarbageCollector {
            objects: LinkedList::new(),
            roots: HashSet::new(),
        }
    }

    fn allocate(&mut self) -> Rc<RefCell<VMObject>> {
        let new_object = Rc::new(RefCell::new(VMObject {
            value: VMValue::Null,
            marked: false,
        }));

        self.objects.push_back(Rc::clone(&new_object));
        new_object
    }

    fn add_root(&mut self, root: &Rc<RefCell<VMObject>>) {
        self.roots.insert(WeakObjectRef(Rc::downgrade(root)));
    }

    fn collect(&mut self) {
        self.mark_phase();
        self.sweep_phase();
    }

    fn mark_phase(&mut self) {
        let roots_clone: HashSet<_> = self.roots.clone();

        for root in &roots_clone {
            if let Some(root_strong) = root.0.upgrade() {
                self.mark_recursive(&root_strong);
            }
        }
    }

    fn mark_recursive(&mut self, obj: &Rc<RefCell<VMObject>>) {
        if !obj.borrow().marked {
            obj.borrow_mut().marked = true;
            // Recursively mark referenced objects if needed
        }
    }

    fn sweep_phase(&mut self) {
        let mut new_objects = LinkedList::new();

        for obj in &self.objects {
            if obj.borrow().marked {
                new_objects.push_back(Rc::clone(obj));
            }
        }

        self.objects = new_objects;
    }
}

// struct CallFrame {
//     ip: usize,
//     locals: Vec<u64>,
// }

struct VMState {
    gc: GarbageCollector,
    // call_stack: VecDeque<CallFrame>,
}

impl VMState {
    fn new() -> VMState {
        VMState {
            gc: GarbageCollector::new(),
            // call_stack: VecDeque::new(),
        }
    }
}

pub struct VM {
    state: VMState,
    module: Module,
    globals: Vec<u64>,
    jc: JitCompiler,
}

impl VM {
    pub fn new(module: Module) -> Self {
        Self {
            state: VMState::new(),
            module,
            globals: Vec::new(),
            jc: JitCompiler::new(),
        }
    }

    fn store_name(&mut self, name_index: u32, obj: Object) -> Object {
        if self.globals.len() <= name_index as usize {
            self.globals.resize(self.module.name_pool.len(), 0);
        }
        self.globals[name_index as usize] = obj.encode();
        obj
    }

    fn load_name(&mut self, name_index: u32) -> Object {
        if self.globals.len() <= name_index as usize {
            panic!("Invalid name index: {}", name_index);
        }
        Object::from(self.globals[name_index as usize])
    }

    fn rust_add(&mut self, lhs: Object, rhs: Object) -> Object {
        assert_eq!(lhs.tag(), rhs.tag());

        match lhs.tag() {
            ObjectTag::Int => {
                return Object::new(lhs.value() + rhs.value(), ObjectTag::Int);
            },
            _ => {
                unimplemented!("unimplemented add for {:?}", lhs.tag());
            }
        }
    }

    fn rust_lt(&mut self, lhs: Object, rhs: Object) -> Object {
        assert_eq!(lhs.tag(), rhs.tag());

        match lhs.tag() {
            ObjectTag::Int => {
                return Object::new((lhs.value() < rhs.value()) as u64, ObjectTag::Int);
            },
            _ => {
                unimplemented!("unimplemented add for {:?}", lhs.tag());
            }
        }
    }

    pub fn run(&mut self) {
        let mut i = 0;

        while false && i < self.module.entry.code.len() {
            let offset = i as u32;
            let opcode = Opcode::try_from(self.module.entry.read8(&mut i)).unwrap();

            match opcode {
                Opcode::JUMP => {
                    let jump_offset = self.module.entry.read32(&mut i);
                    self.jc.compile_jump(offset, jump_offset);
                },
                Opcode::POP_JUMP_IF_FALSE => {
                    let jump_offset = self.module.entry.read32(&mut i);
                    self.jc.compile_pop_jump_if_false(offset, jump_offset);
                },
                Opcode::POP_TOP => {
                    self.jc.compile_pop(offset, Register::RAX);
                },
                Opcode::LOAD_CONST => {
                    let index = self.module.entry.read32(&mut i);
                    let constant = &self.module.constant_pool[index as usize];

                    let mut obj = Object::new(0, ObjectTag::Null);
                    match constant {
                        Constant::Null => {
                            obj = Object::new(0, ObjectTag::Null);
                        },
                        Constant::Int(int) => {
                            obj = Object::new(*int as u64, ObjectTag::Int);
                        }
                        Constant::String(string) => {
                            let heap_obj = self.state.gc.allocate();
                            heap_obj.borrow_mut().value = VMValue::String(string.clone());
                            self.state.gc.add_root(&heap_obj);
                            obj = Object::new(heap_obj.as_ptr() as u64, ObjectTag::Pointer);
                        }
                        Constant::Code(code) => {
                            let heap_obj = self.state.gc.allocate();
                            heap_obj.borrow_mut().value = VMValue::Code(code.clone());
                            self.state.gc.add_root(&heap_obj);
                            obj = Object::new(heap_obj.as_ptr() as u64, ObjectTag::Pointer);
                        }
                        _ => {
                            unimplemented!("Unimplemented constant: {:?}", constant);
                        }
                    }

                    self.jc.compile_push_immediate(offset, Immediate64(obj.encode()));
                },
                Opcode::STORE_NAME => {
                    let index = self.module.entry.read32(&mut i);
                    self.jc.compile_pop(offset, Register::R8);

                    self.jc.compile_call(
                        offset,
                        VM::store_name as u64,
                        vec![
                            Operand::Immediate(Immediate::Immediate64(self as *const _ as u64)),
                            Operand::Immediate(Immediate::Immediate32(index)),
                            Operand::Register(Register::R8, false),
                        ]
                    );
                },
                Opcode::LOAD_NAME => {
                    let index = self.module.entry.read32(&mut i);

                    self.jc.compile_call(
                        offset,
                        VM::load_name as u64,
                        vec![
                            Operand::Immediate(Immediate::Immediate64(self as *const _ as u64)),
                            Operand::Immediate(Immediate::Immediate32(index)),
                        ]
                    );

                    self.jc.compile_push(offset, Operand::Register(Register::RAX, false));
                },
                Opcode::RETURN => {
                    self.jc.compile_return(offset);
                },
                Opcode::BINARY_ADD => {
                    self.jc.compile_pop(offset, Register::RDX);
                    self.jc.compile_pop(offset, Register::R8);

                    self.jc.compile_call(
                        offset,
                        VM::rust_add as u64,
                        vec![
                            Operand::Immediate(Immediate::Immediate64(self as *const _ as u64)),
                            Operand::Register(Register::RDX, false),
                            Operand::Register(Register::R8, false),
                        ]
                    );

                    self.jc.compile_push(offset, Operand::Register(Register::RAX, false));
                },
                Opcode::BINARY_BOOLEAN_LESS_THAN => {
                    self.jc.compile_pop(offset, Register::R8);
                    self.jc.compile_pop(offset, Register::RDX);

                    // check if tag == Int for fastpath


                    self.jc.compile_call(
                        offset,
                        VM::rust_lt as u64,
                        vec![
                            Operand::Immediate(Immediate::Immediate64(self as *const _ as u64)),
                            Operand::Register(Register::RDX, false),
                            Operand::Register(Register::R8, false),
                        ]
                    );

                    // end:
                    self.jc.compile_push(offset, Operand::Register(Register::RAX, false));
                },
                Opcode::CALL => {
                    let arg_size = self.module.entry.read32(&mut i) as usize;
                    let integer_regs = vec![
                        Register::RCX,
                        Register::RDX,
                        Register::R8,
                        Register::R9
                    ];

                    assert!(arg_size <= integer_regs.len());

                    for i in 0..arg_size {
                        self.jc.compile_pop(offset, integer_regs[i]);
                    }

                    self.jc.compile_pop(offset, Register::RAX);
                    // code object should be in RAX
                    /*
                    if object.tag() == CodeObject { check in jitted functions, if not then jit it }
                    else if object.tag() == NativeFuncObject { call object.value()() }
                    else { error idfk }
                    */

                    // self.jc.compile_shr(Operand::Immediate(Immediate8(48)));
                    // self.jc.compile_cmp(Operand::Register(Register::RAX),Operand::Immediate(Immediate8(ObjectTag::Code)))
                },
                _ => {
                    unimplemented!("Unimplemented opcode: {:?}", opcode);
                }
            }
        }

        self.jc.compile();
        let obj = Object::from(self.jc.run());
        match obj.tag() {
            ObjectTag::Null => {
                println!("null");
            }
            ObjectTag::Int => {
                dbg!(&obj.value());
            },
            ObjectTag::Pointer => {
                let vm_obj = unsafe {
                    let raw_ptr = obj.value() as *mut VMObject;
                    &mut *raw_ptr
                };
                match &vm_obj.value {
                    VMValue::String(x) => {
                        dbg!(&x);
                    },
                    _ => {
                        unimplemented!("Unimplemented object tag: {:?}", obj.tag());
                    }
                };
            },
            _ => {
                unimplemented!("Unimplemented object tag: {:?}", obj.tag());
            }
        }

        self.state.gc.collect();
    }
}