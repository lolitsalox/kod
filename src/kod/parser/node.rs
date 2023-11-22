use std::fmt::Debug;

use crate::kod::lexer::token::{TokenType, get_symbols};
use crate::kod::compiler::bytekod::{Code, Module, Opcode, Constant};

pub trait Node: Debug {
    fn to_string(&self) -> String;
    fn pushes(&self) -> bool {
        true
    }
    fn returns(&self) -> bool {
        false
    }
    fn is_constant(&self) -> bool {
        false
    }

    fn get_int_mut(&mut self) -> Option<&mut IntNode> {
        None
    }

    fn get_access(&self) -> Option<&AccessNode> {
        None
    }

    fn get_access_mut(&mut self) -> Option<&mut AccessNode> {
        None
    }

    fn get_id(&self) -> Option<&IdNode> {
        None
    }

    fn take_id(self: Box<Self>) -> Option<IdNode> {
        None
    }

    fn get_block(&self) -> Option<&BlockNode> {
        None
    }

    fn take_block(self: Box<Self>) -> Option<BlockNode> {
        None
    }
    
    fn get_tuple_mut(&mut self) -> Option<&mut TupleNode> {
        None
    }

    fn get_tuple(&self) -> Option<&TupleNode> {
        None
    }

    fn take_tuple(self: Box<Self>) -> Option<TupleNode> {
        None
    }

    fn get_float_mut(&mut self) -> Option<&mut FloatNode> {
        None
    }

    fn get_string_mut(&mut self) -> Option<&mut StringNode> {
        None
    }

    fn compile(&self, module: &mut Module, code: &mut Code) {
        unimplemented!("Unimplemented compile: {}", self.to_string());
    }

    fn push(&self, module: &mut Module, code: &mut Code) {
        unimplemented!("Unimplemented push: {}", self.to_string());
    }

    fn to_constant(&self) -> Constant {
        unimplemented!("Unimplemented to_constant: {}", self.to_string());
    }

    fn get(&self) -> NodeEnum {
        unimplemented!("Unimplemented get: {}", self.to_string());
    }

    // fn compile(&self, module: &mut CompiledModule, code: &mut Code) {
    //     panic!("Unimplemented compile: {}", self.to_string());
    // }

    // fn to_constant(&self) -> Constant {
    //     panic!("Unimplemented to_constant: {}", self.to_string());
    // }

    // fn push(&self, module: &mut CompiledModule, code: &mut Code) {
    //     panic!("Unimplemented push: {}", self.to_string());
    // }
}

#[derive(Debug)]
pub enum NodeEnum<'a> {
    Block(&'a BlockNode),
    FuncDef(&'a FuncDefNode),
    FuncCall(&'a FuncCallNode),
    Assignment(&'a AssignmentNode),
    BinaryOp(&'a BinaryOpNode),
    UnaryOp(&'a UnaryOpNode),
    Access(&'a AccessNode),
    Id(&'a IdNode),
    Tuple(&'a TupleNode),
    Float(&'a FloatNode),
    String(&'a StringNode),
    Int(&'a IntNode),
    Subscript(&'a SubscriptNode),
    Return(&'a ReturnNode),
    If(&'a IfNode),
    While(&'a WhileNode),
}

#[derive(Debug)]
pub struct BlockNode {
    pub statements: Vec<Box<dyn Node>>,
}

#[derive(Debug)]
pub struct FuncDefNode {
    pub name: Box<IdNode>,
    pub params: Vec<Box<dyn Node>>,
    pub body: Box<BlockNode>,
}

#[derive(Debug)]
pub struct FuncCallNode {
    pub callee: Box<dyn Node>,
    pub args: Vec<Box<dyn Node>>,
    pub add_arg: bool,
}

#[derive(Debug)]
pub struct AssignmentNode {
    pub left: Box<dyn Node>,
    pub right: Box<dyn Node>,
    pub op: TokenType,
}

#[derive(Debug)]
pub struct BinaryOpNode {
    pub left: Box<dyn Node>,
    pub right: Box<dyn Node>,
    pub op: TokenType,
}

#[derive(Debug)]
pub struct UnaryOpNode {
    pub value: Box<dyn Node>,
    pub op: TokenType,
}

#[derive(Debug)]
pub struct AccessNode {
    pub value: Box<dyn Node>,
    pub field: Box<dyn Node>,
    pub load_self: bool,
}

#[derive(Debug)]
pub struct SubscriptNode {
    pub value: Box<dyn Node>,
    pub subscript: Box<dyn Node>,
}

#[derive(Debug)]
pub struct ReturnNode {
    pub value: Option<Box<dyn Node>>,
}

#[derive(Debug)]
pub struct IfNode {
    pub condition: Box<dyn Node>,
    pub block: Box<BlockNode>,
}

#[derive(Debug)]
pub struct WhileNode {
    pub condition: Box<dyn Node>,
    pub block: Box<BlockNode>,
}

#[derive(Debug)]
pub struct TupleNode {
    pub values: Vec<Box<dyn Node>>,
    pub is_list: bool,
}

#[derive(Debug)]
pub struct IntNode {
    pub value: i64,
}

#[derive(Debug)]
pub struct FloatNode {
    pub value: f64,
}

#[derive(Debug)]
pub struct StringNode {
    pub value: String,
}

#[derive(Debug)]
pub struct IdNode {
    pub value: String,
}

impl Node for BlockNode {
    fn to_string(&self) -> String {
        let mut s = String::new();
        for statement in &self.statements {
            s.push_str(&statement.to_string());
            s.push('\n');
        }
        s
    }

    fn get_block(&self) -> Option<&BlockNode> {
        Some(self)
    }

    fn take_block(self: Box<Self>) -> Option<BlockNode> {
        Some(*self)
    }

    fn compile(&self, module: &mut Module, code: &mut Code) {
        code.code.clear();

        for (index, statement) in self.statements.iter().enumerate() {
            statement.compile(module, code);
            if !statement.pushes() { continue; }

            if index != self.statements.len() - 1 {
                code.code.push(Opcode::POP_TOP.encode());
            }
        }

        if self.statements.is_empty() || (self.statements.len() > 0 && !self.statements[self.statements.len() - 1].returns()) {
            if self.statements.len() == 0 || (self.statements.len() > 0 && !self.statements[self.statements.len() - 1].pushes()) {
                // find the index of the constant null in module.constant_pool
                let constant = module.constant_pool.iter().enumerate().find(|(_, constant)| {
                    match constant {
                        Constant::Null => true,
                        _ => false
                    }
                });

                code.emit8(Opcode::LOAD_CONST.encode());

                match constant {
                    Some((index, _)) => {
                        code.emit32(index as u32);
                    }
                    None => {
                        module.constant_pool.push(Constant::Null);
                        code.emit32(module.constant_pool.len() as u32 - 1);
                    }
                }
            }
            
            code.emit8(Opcode::RETURN.encode()); // implement RETURN_CONST and RETURN_VALUE idk
        }
    }

    fn get(&self) -> NodeEnum {
        NodeEnum::Block(self)
    }
}

fn assign(module: &mut Module, code: &mut Code, left: &Box<dyn Node>, right: Option<&Box<dyn Node>>, op: TokenType) {
    if right.is_some() {
        let right = right.unwrap();
        right.compile(module, code);
        if !right.pushes() {
            right.push(module, code);
        }
    }

    if let Some(identifier) = left.get_id() {
        let id = module.name_pool.iter().enumerate().find(|(_, id)| {
            **id == identifier.value
        });

        match op {
            TokenType::EQUALS => {
                code.emit8(Opcode::STORE_NAME.encode());
            },
            _ => {
                unimplemented!("Unimplemented assignment operator: {:?}", op);
            }
        };

        match id {
            Some((index, _)) => {
                code.emit32(index as u32);
            }
            None => {
                module.name_pool.push(identifier.value.clone());
                code.emit32(module.name_pool.len() as u32 - 1);
            }
        };

    } else if let Some(tuple) = left.get_tuple() {
        code.emit8(Opcode::UNPACK_SEQUENCE.encode());
        code.emit32(tuple.values.len() as u32);

        for value in &tuple.values {
            assign(module, code, &value, None, op);
        }
    } else {
        unimplemented!("Unimplemented assignment for {:?}", left);
    }
}

impl Node for AssignmentNode {
    fn to_string(&self) -> String {
        return format!("{} {} {}", self.left.to_string(), get_symbols().iter().find(|x| x.1 == &self.op).unwrap().0, self.right.to_string());
    }

    fn compile(&self, module: &mut Module, code: &mut Code) {
        assign(module, code, &self.left, Some(&self.right), self.op);
    }

    fn pushes(&self) -> bool {
        false
    }

    fn push(&self, module: &mut Module, code: &mut Code) {
        if let Some(name) = self.left.get_id() {
            let id = module.name_pool.iter().enumerate().find(|(_, id)| {
                **id == name.value
            });

            code.emit8(Opcode::LOAD_NAME.encode());

            match id {
                Some((index, _)) => {
                    code.emit32(index as u32);
                }
                None => {
                    module.name_pool.push(name.value.clone());
                    code.emit32(module.name_pool.len() as u32 - 1);
                }
            };
        } else {
            unimplemented!("Unimplemented push for {:?}", self.left);
        }
    }

    fn get(&self) -> NodeEnum {
        NodeEnum::Assignment(self)
    }
}

impl Node for BinaryOpNode {
    fn to_string(&self) -> String {
        return format!("({} {} {})", self.left.to_string(), get_symbols().iter().find(|x| x.1 == &self.op).unwrap().0, self.right.to_string());
    }

    fn compile(&self, module: &mut Module, code: &mut Code) {
        self.left.compile(module, code);
        self.right.compile(module, code);
    
        let bin_op = match self.op {
            TokenType::ADD => Opcode::BINARY_ADD,
            TokenType::SUB => Opcode::BINARY_SUB,
            TokenType::MUL => Opcode::BINARY_MUL,
            TokenType::DIV => Opcode::BINARY_DIV,
            TokenType::MOD => Opcode::BINARY_MOD,
            TokenType::POW => Opcode::BINARY_POW,
            TokenType::AND => Opcode::BINARY_AND,
            TokenType::OR => Opcode::BINARY_OR,
            TokenType::HAT => Opcode::BINARY_XOR,
            TokenType::SHL => Opcode::BINARY_LEFT_SHIFT,
            TokenType::SHR => Opcode::BINARY_RIGHT_SHIFT,
            TokenType::BoolAnd => Opcode::BINARY_BOOLEAN_AND,
            TokenType::BoolOr => Opcode::BINARY_BOOLEAN_OR,
            TokenType::BoolEq => Opcode::BINARY_BOOLEAN_EQUAL,
            TokenType::BoolNe => Opcode::BINARY_BOOLEAN_NOT_EQUAL,
            TokenType::BoolGt => Opcode::BINARY_BOOLEAN_GREATER_THAN,
            TokenType::BoolGte => Opcode::BINARY_BOOLEAN_GREATER_THAN_OR_EQUAL_TO,
            TokenType::BoolLt => Opcode::BINARY_BOOLEAN_LESS_THAN,
            TokenType::BoolLte => Opcode::BINARY_BOOLEAN_LESS_THAN_OR_EQUAL_TO,
            _ => unreachable!("Invalid binary operator: {:?}", self.op),
        };
    
        code.emit8(bin_op.encode());
    }

    fn is_constant(&self) -> bool {
        self.left.is_constant() && self.right.is_constant()
    }

    fn get(&self) -> NodeEnum {
        NodeEnum::BinaryOp(self)
    }
}

impl Node for UnaryOpNode {
    fn to_string(&self) -> String {
        return format!("({}{})", get_symbols().iter().find(|x| x.1 == &self.op).unwrap().0, self.value.to_string());
    }



    fn is_constant(&self) -> bool {
        self.value.is_constant()
    }

    fn get(&self) -> NodeEnum {
        NodeEnum::UnaryOp(self)
    }
}

impl Node for ReturnNode {
    fn to_string(&self) -> String {
        return format!("return {}", if self.value.is_some() { self.value.as_deref().unwrap().to_string() } else { "null".to_string() });
    }

    fn compile(&self, module: &mut Module, code: &mut Code) {
        if let Some(value) = &self.value {
            value.compile(module, code);
            if !value.pushes() {
                value.push(module, code);
            }
        } else {
            load_null(module, code);
        }

        code.emit8(Opcode::RETURN.encode());
    }

    fn pushes(&self) -> bool {
        false
    }

    fn returns(&self) -> bool {
        true
    }

    fn get(&self) -> NodeEnum {
        NodeEnum::Return(self)
    }
}

impl Node for AccessNode {
    fn to_string(&self) -> String {
        return format!("{}.{}", self.value.to_string(), self.field.to_string());
    }

    fn get_access(&self) -> Option<&AccessNode> {
        Some(self)
    }

    fn get_access_mut(&mut self) -> Option<&mut AccessNode> {
        Some(self)
    }

    fn get(&self) -> NodeEnum {
        NodeEnum::Access(self)
    }
}

impl Node for SubscriptNode {
    fn to_string(&self) -> String {
        return format!("{}[{}]", self.value.to_string(), self.subscript.to_string());
    }

    fn get(&self) -> NodeEnum {
        NodeEnum::Subscript(self)
    }
}

impl Node for IfNode {
    fn to_string(&self) -> String {
        return format!("if {} {{\n{}}}", self.condition.to_string(), self.block.to_string());
    }

    fn compile(&self, module: &mut Module, code: &mut Code) {
        self.condition.compile(module, code);
        if !self.condition.pushes() {
            self.condition.push(module, code);
        }

        code.emit8(Opcode::POP_JUMP_IF_FALSE.encode());
        let end_offset = code.emit32(0);

        for statement in &self.block.statements {
            statement.compile(module, code);
            if !statement.pushes() { continue; }
            code.emit8(Opcode::POP_TOP.encode());
        }

        code.patch32(end_offset, code.code.len() as u32);
    }

    fn pushes(&self) -> bool {
        false
    }

    fn get(&self) -> NodeEnum {
        NodeEnum::If(self)
    }
}

impl Node for WhileNode {
    fn to_string(&self) -> String {
        return format!("while {} {{\n{}}}", self.condition.to_string(), self.block.to_string());
    }

    fn compile(&self, module: &mut Module, code: &mut Code) {
        let condition_offset = code.code.len();
        
        self.condition.compile(module, code);
        if !self.condition.pushes() {
            self.condition.push(module, code);
        }

        code.emit8(Opcode::POP_JUMP_IF_FALSE.encode());
        let end_offset = code.emit32(0);

        for statement in &self.block.statements {
            statement.compile(module, code);
            if !statement.pushes() { continue; }
            code.emit8(Opcode::POP_TOP.encode());
        }

        code.emit8(Opcode::JUMP.encode());
        code.emit32(condition_offset as u32);

        code.patch32(end_offset, code.code.len() as u32);
    }

    fn pushes(&self) -> bool {
        false
    }

    fn get(&self) -> NodeEnum {
        NodeEnum::While(self)
    }
}

impl Node for TupleNode {
    fn to_string(&self) -> String {
        let delims = if self.is_list {
            ("[", "]")
        } else {
            ("(", ")")
        };
        return delims.0.to_string() + &self.values.iter().map(|x| { x.to_string() }).collect::<Vec<String>>().join(", ") + delims.1;
    }

    fn get_tuple_mut(&mut self) -> Option<&mut TupleNode> {
        Some(self)
    }

    fn get_tuple(&self) -> Option<&TupleNode> {
        Some(self)
    }
    
    fn take_tuple(self: Box<Self>) -> Option<TupleNode> {
        Some(*self)
    }



    fn is_constant(&self) -> bool {
        self.values.iter().all(|x| { x.is_constant() })
    }

    fn to_constant(&self) -> Constant {
        Constant::Tuple(self.values.iter().map(|x| { x.to_constant() }).collect())
    }

    fn get(&self) -> NodeEnum {
        NodeEnum::Tuple(self)
    }
}

impl Node for IntNode {
    fn to_string(&self) -> String {
        return self.value.to_string();
    }

    fn is_constant(&self) -> bool {
        return true;
    }

    fn get_int_mut(&mut self) -> Option<&mut IntNode> {
        Some(self)
    }

    fn compile(&self, module: &mut Module, code: &mut Code) {
        let constant = module.constant_pool.iter().enumerate().find(|(_, constant)| {
            match constant {
                Constant::Int(x) => x == &self.value,
                _ => false
            }
        });
    
        code.emit8(Opcode::LOAD_CONST.encode());
        
        match constant {
            Some((index, _)) => {
                code.emit32(index as u32);
            }
            None => {
                module.constant_pool.push(Constant::Int(self.value));
                code.emit32(module.constant_pool.len() as u32 - 1);
            }
        }   
    }

    fn to_constant(&self) -> Constant {
        Constant::Int(self.value)
    }

    fn get(&self) -> NodeEnum {
        NodeEnum::Int(self)
    }
}

impl Node for FloatNode {
    fn to_string(&self) -> String {
        return self.value.to_string();
    }

    fn is_constant(&self) -> bool {
        return true;
    }

    fn get_float_mut(&mut self) -> Option<&mut FloatNode> {
        Some(self)
    }



    fn to_constant(&self) -> Constant {
        Constant::Float(self.value)
    }

    fn get(&self) -> NodeEnum {
        NodeEnum::Float(self)
    }
}

impl Node for StringNode {
    fn to_string(&self) -> String {
        return format!("\"{}\"", self.value);
    }

    fn is_constant(&self) -> bool {
        return true;
    }

    fn get_string_mut(&mut self) -> Option<&mut StringNode> {
        Some(self)
    }

    fn compile(&self, module: &mut Module, code: &mut Code) {
        let constant = module.constant_pool.iter().enumerate().find(|(_, constant)| {
            match constant {
                Constant::String(x) => x == &self.value,
                _ => false
            }
        });
    
        code.emit8(Opcode::LOAD_CONST.encode());
        
        match constant {
            Some((index, _)) => {
                code.emit32(index as u32);
            }
            None => {
                module.constant_pool.push(Constant::String(self.value.clone()));
                code.emit32(module.constant_pool.len() as u32 - 1);
            }
        }   
    }

    fn to_constant(&self) -> Constant {
        Constant::String(self.value.clone())
    }

    fn get(&self) -> NodeEnum {
        NodeEnum::String(self)
    }
}

impl Node for IdNode {
    fn to_string(&self) -> String {
        return self.value.clone();
    }

    fn get_id(&self) -> Option<&IdNode> {
        Some(self)
    }

    fn take_id(self: Box<Self>) -> Option<IdNode> {
        Some(*self)
    }

    fn compile(&self, module: &mut Module, code: &mut Code) {
        let id = module.name_pool.iter().enumerate().find(|(_, id)| {
            **id == self.value
        });

        code.emit8(Opcode::LOAD_NAME.encode());

        match id {
            Some((index, _)) => {
                code.emit32(index as u32);
            }
            None => {
                module.name_pool.push(self.value.clone());
                code.emit32(module.name_pool.len() as u32 - 1);
            }
        }
    }

    fn get(&self) -> NodeEnum {
        NodeEnum::Id(self)
    }
}

fn load_null(module: &mut Module, code: &mut Code) {
    let constant = module.constant_pool.iter().enumerate().find(|(_, constant)| {
        match constant {
            Constant::Null => true,
            _ => false
        }
    });

    code.emit8(Opcode::LOAD_CONST.encode());
    
    match constant {
        Some((index, _)) => {
            code.emit32(index as u32);
        }
        None => {
            module.constant_pool.push(Constant::Null);
            code.emit32(module.constant_pool.len() as u32 - 1);
        }
    }   
}

fn compile_code_constant(
    module: &mut Module, 
    code: &mut Code,
    name: &String,
    func_args: &Vec<Box<dyn Node>>,
    func_body: &Box<BlockNode>
) {
    let mut func = Code::new(name.clone(), vec![], vec![]);

    func.params = func_args.iter().map(|x| { x.to_string() }).collect();
    for node in &func_body.statements {
        node.compile(module, &mut func);
        if !node.pushes() { continue; }
        func.emit8(Opcode::POP_TOP.encode());
    }

    if func_body.statements.is_empty() || (func_body.statements.len() > 0 && !func_body.statements.last().unwrap().returns()) {
        load_null(module, &mut func);
    }

    let func_constant = Constant::Code(func);
    let constant = module.constant_pool.iter().enumerate().find(|(_, constant)| {
        func_constant == **constant
    });

    code.emit8(Opcode::LOAD_CONST.encode());

    match constant {
        Some((index, _)) => {
            code.emit32(index as u32);
        }
        None => {
            module.constant_pool.push(func_constant);
            code.emit32(module.constant_pool.len() as u32 - 1);
        }
    }
}

impl Node for FuncDefNode {
    fn to_string(&self) -> String {
        return format!("{}({}) {{\n{}}}", self.name.to_string(), self.params.iter().map(|x| { x.to_string() }).collect::<Vec<String>>().join(", "), self.body.to_string());
    }

    fn compile(&self, module: &mut Module, code: &mut Code) {
        let name = self.name.to_string();

        compile_code_constant(module, code, &name, &self.params, &self.body);

        let func_name = module.name_pool.iter().enumerate().find(|(_, func_name)| {
            **func_name == name
        });

        code.emit8(Opcode::STORE_NAME.encode());

        match func_name {
            Some((index, _)) => {
                code.emit32(index as u32);
            }
            None => {
                module.name_pool.push(name);
                code.emit32(module.name_pool.len() as u32 - 1);
            }
        }
    }

    fn pushes(&self) -> bool {
        false
    }

    fn get(&self) -> NodeEnum {
        NodeEnum::FuncDef(self)
    }
}

impl Node for FuncCallNode {
    fn to_string(&self) -> String {
        return format!("{}({})", self.callee.to_string(), self.args.iter().map(|x| { x.to_string() }).collect::<Vec<String>>().join(", "));
    }

    fn compile(&self, module: &mut Module, code: &mut Code) {
        self.callee.compile(module, code);

        self.args.iter().map(|x| { x.compile(module, code); }).count();

        code.emit8(Opcode::CALL.encode());
        code.emit32(self.args.len() as u32 + if self.add_arg { 1 } else { 0 });
    }

    fn get(&self) -> NodeEnum {
        NodeEnum::FuncCall(self)
    }
}