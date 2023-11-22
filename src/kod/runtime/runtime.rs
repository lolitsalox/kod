use crate::kod::{compiler::{bytekod::{Code, Module, Opcode, Constant}, compiler::JitCompiler, assembler::{Operand, Register, Immediate, Label, Condition}}, parser::node::{Node, IntNode, NodeEnum}, lexer::token::TokenType};

#[derive(Debug, Copy, Clone)]
pub struct Object(u64);


#[derive(Debug, Copy, Clone)]
pub enum ObjectTag {
    Int,
    Null,
    Float,
    Pointer,
}

impl ObjectTag {
    pub fn from(value: u16) -> Self {
        match value {
            0 => Self::Int,
            1 => Self::Null,
            2 => Self::Float,
            3 => Self::Pointer,
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

use std::{cell::RefCell, collections::VecDeque};
use std::collections::{HashSet, LinkedList};
use std::hash::{Hash, Hasher};
use std::rc::{Rc, Weak};

#[derive(Debug)]
enum VMValue {
    Null,
    String(String),
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

    fn compile(&mut self, node: &Box<dyn Node>) {
        match node.get() {
            NodeEnum::Int(node) => {
                let obj = Object::new(node.value as u64, ObjectTag::Int);
                self.jc.assembler.mov(
                    Operand::Register(Register::RAX, false), 
                    Operand::Immediate(Immediate::Immediate64(obj.encode()))
                );
            },
            NodeEnum::Return(node) => {
                if node.value.is_some() {
                    self.compile(&node.value.as_ref().unwrap());
                }
                self.jc.assembler.exit();
            }
            NodeEnum::BinaryOp(node) => {
                // Compile the left operand
                self.compile(&node.left);

                // Move the left operand to a temporary register (let's use RCX)
                self.jc.assembler.mov(
                    Operand::Register(Register::RCX, false),
                    Operand::Register(Register::RAX, false),
                );

                // Evaluate the right operand (assuming it's on top of the stack)
                self.compile(&node.right);

                // Perform the binary operation using the left operand in RCX and the right operand in RAX
                match node.op {
                    TokenType::ADD => {
                        self.jc.assembler.add(
                            Operand::Register(Register::RCX, false),
                            Operand::Register(Register::RAX, false),
                        );
                    },
                    TokenType::BoolLt => {
                        // make a label for true and a label for falce
                        let mut true_label = Label::new();

                        self.jc.assembler.jump_if_cmp(
                            &Operand::Register(Register::RCX, false),
                            Condition::SignedLessThan,
                            &Operand::Register(Register::RAX, false),
                            &mut true_label,
                        );

                        let mut end_label = Label::new();
                        self.jc.assembler.mov(
                            Operand::Register(Register::RCX, false),
                            Operand::Immediate(Immediate::Immediate64(0)),
                        );
                        
                        self.jc.assembler.jump_label(&mut end_label);
                        true_label.link(&mut self.jc.assembler);
                        self.jc.assembler.mov(
                            Operand::Register(Register::RCX, false),
                            Operand::Immediate(Immediate::Immediate64(1)),
                        );
                        end_label.link(&mut self.jc.assembler);
                    }
                    // Handle other binary operations...
                    _ => {
                        unimplemented!("Unimplemented binary op: {:?}", node.op);
                    }
                }

                // Move the result back to RAX if needed
                self.jc.assembler.mov(
                    Operand::Register(Register::RAX, false),
                    Operand::Register(Register::RCX, false),
                );
            },
            NodeEnum::Id(node) => {
                let index = self.module.name_pool.iter().position(|x| **x == node.value).unwrap() as u32;

                self.jc.assembler.mov(Operand::Register(Register::RCX, false), Operand::Immediate(Immediate::Immediate64(self as *const _ as u64)));
                self.jc.assembler.mov(Operand::Register(Register::RDX, false), Operand::Immediate(Immediate::Immediate32(index)));
                self.jc.assembler.mov(Operand::Register(Register::RAX, false), Operand::Immediate(Immediate::Immediate64(VM::load_name as u64)));
                self.jc.assembler.call_rax();
            },
            NodeEnum::Assignment(node) => {
                self.compile(&node.right);
                // store RAX in R8
                self.jc.assembler.mov(
                    Operand::Register(Register::R8, false),
                    Operand::Register(Register::RAX, false),
                );

                if node.left.get_id().is_some() {
                    let index = self.module.name_pool.iter().position(|x| **x == node.left.get_id().unwrap().value).unwrap() as u32;
                    self.jc.assembler.mov(Operand::Register(Register::RCX, false), Operand::Immediate(Immediate::Immediate64(self as *const _ as u64)));
                    self.jc.assembler.mov(Operand::Register(Register::RDX, false), Operand::Immediate(Immediate::Immediate32(index)));
                    self.jc.assembler.mov(Operand::Register(Register::RAX, false), Operand::Immediate(Immediate::Immediate64(VM::store_name as u64)));
                    self.jc.assembler.call_rax();
                } else {
                    unimplemented!("Unimplemented assignment for {:?}", node.left);
                }
            },
            NodeEnum::While(node) => {
                let mut cond_label = Label::new();
                let mut end_label = Label::new();
                cond_label.link(&mut self.jc.assembler);
                self.jc.assembler.trap();

                self.compile(&node.condition);
                self.jc.assembler.jump_if_cmp(
                    &Operand::Register(Register::RAX, false), 
                    Condition::SignedLessThanOrEqualTo, 
                    &Operand::Immediate(Immediate::Immediate32(0)), 
                    &mut end_label
                );

                for st in &node.block.statements {
                    self.compile(st);
                }
                self.jc.assembler.jump_label(&mut cond_label);

                end_label.link(&mut self.jc.assembler);                
            },
            _ => {
                unimplemented!("Unimplemented node: {:?}", node);
            }
        }
        
    }

    pub fn run(&mut self, node: &Box<dyn Node>) {
        self.jc.assembler.enter();
        for st in &node.get_block().unwrap().statements {
            self.compile(st);
        }

        // let mut i = 0;
        // while i < self.module.entry.code.len() {
        //     let opcode = Opcode::try_from(self.module.entry.read8(&mut i)).unwrap();

        //     match opcode {
        //         Opcode::LOAD_CONST => {
        //             let index = self.module.entry.read32(&mut i);
        //             let constant = &self.module.constant_pool[index as usize];

        //             let mut obj = Object::new(0, ObjectTag::Null);
        //             match constant {
        //                 Constant::Null => {
        //                     obj = Object::new(0, ObjectTag::Null);
        //                 },
        //                 Constant::Int(x) => {
        //                     obj = Object::new(*x as u64, ObjectTag::Int);       
        //                 },
        //                 Constant::String(x) => {    
        //                     let heap_obj = self.state.gc.allocate();
        //                     heap_obj.borrow_mut().value = VMValue::String(x.clone());
        //                     self.state.gc.add_root(&heap_obj);
        //                     obj = Object::new(heap_obj.as_ptr() as u64, ObjectTag::Pointer);
        //                 },
        //                 _ => {
        //                     unimplemented!("Unimplemented constant: {:?}", constant);
        //                 }
        //             }

        //             self.jc.assembler.mov(
        //                 Operand::Register(Register::RAX, false),
        //                 Operand::Immediate(Immediate::Immediate64(obj.encode())),
        //             );
        //             self.jc.assembler.push(Operand::Register(Register::RAX, false));
        //         },
        //         Opcode::STORE_NAME => {
        //             let index = self.module.entry.read32(&mut i);
        //             self.jc.assembler.pop(Operand::Register(Register::R8, false));
                    
        //             self.jc.assembler.mov(Operand::Register(Register::RCX, false), Operand::Immediate(Immediate::Immediate64(self as *const _ as u64)));
        //             self.jc.assembler.mov(Operand::Register(Register::RDX, false), Operand::Immediate(Immediate::Immediate32(index)));
        //             self.jc.assembler.mov(Operand::Register(Register::RAX, false), Operand::Immediate(Immediate::Immediate64(VM::store_name as u64)));
        //             self.jc.assembler.call_rax();
        //         },
        //         Opcode::LOAD_NAME => {
        //             let index = self.module.entry.read32(&mut i);
        //             self.jc.assembler.mov(Operand::Register(Register::RCX, false), Operand::Immediate(Immediate::Immediate64(self as *const _ as u64)));
        //             self.jc.assembler.mov(Operand::Register(Register::RDX, false), Operand::Immediate(Immediate::Immediate32(index)));
        //             self.jc.assembler.mov(Operand::Register(Register::RAX, false), Operand::Immediate(Immediate::Immediate64(VM::load_name as u64)));
        //             self.jc.assembler.call_rax();
        //             self.jc.assembler.push(Operand::Register(Register::RAX, false));
        //         },
        //         Opcode::RETURN => {
        //             self.jc.assembler.pop(Operand::Register(Register::RAX, false));
        //             self.jc.assembler.exit();
        //         },
        //         Opcode::BINARY_BOOLEAN_LESS_THAN => {
        //             self.jc.assembler.pop(Operand::Register(Register::RAX, false));
        //             self.jc.assembler.pop(Operand::Register(Register::R8, false));
        //             // simple for now
        //             self.jc.assembler.cmp(&Operand::Register(Register::RAX, false), &Operand::Register(Register::R8, false));
        //             self.jc.assembler.push(Operand::Register(Register::RAX, false));
        //         },
        //         _ => {
        //             unimplemented!("Unimplemented opcode: {:?}", opcode);
        //         }
        //     }
        // }

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