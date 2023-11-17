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
    
    fn get_tuple_mut(&mut self) -> Option<&mut TupleNode> {
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
pub struct AssignmentNode {
    pub left: Box<dyn Node>,
    pub right: Box<dyn Node>,
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
}

impl Node for AssignmentNode {
    fn to_string(&self) -> String {
        return format!("{} = {}", self.left.to_string(), self.right.to_string());
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
        let delims = if self.is_list { ('[', ']') } else { ('(', ')') };
        let mut s = delims.0.to_string();

        for (i, value) in self.values.iter().enumerate() {
            s.push_str(&value.to_string());
            if i < self.values.len() - 1 {
                s.push_str(", ");
            }
        }

        if self.values.len() == 1 {
            s.push(',');
        }
        s.push(delims.1);
        s
    }

    fn get_tuple_mut(&mut self) -> Option<&mut TupleNode> {
        Some(self)
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
        return self.value.clone();
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
}