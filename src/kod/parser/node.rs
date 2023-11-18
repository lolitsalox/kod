use std::fmt::Debug;

use crate::kod::lexer::token::{TokenType, get_symbols};

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
    pub callie: Box<dyn Node>,
    pub args: Vec<Box<dyn Node>>,
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
    pub value: Box<dyn Node>,
}

#[derive(Debug)]
pub struct IfNode {
    pub condition: Box<dyn Node>,
    pub block: Box<dyn Node>,
}

#[derive(Debug)]
pub struct WhileNode {
    pub condition: Box<dyn Node>,
    pub block: Box<dyn Node>,
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

    fn take_block(self: Box<Self>) -> Option<BlockNode> {
        Some(*self)
    }
}

impl Node for AssignmentNode {
    fn to_string(&self) -> String {
        return format!("{} {} {}", self.left.to_string(), get_symbols().iter().find(|x| x.1 == &self.op).unwrap().0, self.right.to_string());
    }
}

impl Node for BinaryOpNode {
    fn to_string(&self) -> String {
        return format!("({} {} {})", self.left.to_string(), get_symbols().iter().find(|x| x.1 == &self.op).unwrap().0, self.right.to_string());
    }
}

impl Node for UnaryOpNode {
    fn to_string(&self) -> String {
        return format!("({}{})", get_symbols().iter().find(|x| x.1 == &self.op).unwrap().0, self.value.to_string());
    }
}

impl Node for ReturnNode {
    fn to_string(&self) -> String {
        return format!("return {}", self.value.to_string());
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
}

impl Node for SubscriptNode {
    fn to_string(&self) -> String {
        return format!("{}[{}]", self.value.to_string(), self.subscript.to_string());
    }
}

impl Node for IfNode {
    fn to_string(&self) -> String {
        return format!("if {}:\n{}", self.condition.to_string(), self.block.to_string());
    }
}

impl Node for WhileNode {
    fn to_string(&self) -> String {
        return format!("while {}:\n{}", self.condition.to_string(), self.block.to_string());
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
}

impl Node for FuncDefNode {
    fn to_string(&self) -> String {
        return format!("{}({}) {{\n{}}}", self.name.to_string(), self.params.iter().map(|x| { x.to_string() }).collect::<Vec<String>>().join(", "), self.body.to_string());
    }
}

impl Node for FuncCallNode {
    fn to_string(&self) -> String {
        return format!("{}({})", self.callie.to_string(), self.args.iter().map(|x| { x.to_string() }).collect::<Vec<String>>().join(", "));
    }
}