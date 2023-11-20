
pub struct Code {
    
}

pub enum Constant {
    Int(i64),
    Float(f64),
    String(String),
    Code(Code),
    Tuple(Vec<Constant>),
}

/*
The module has a couple things:
a name
a constant pool
a name pool
an entry point
*/

pub struct Module {
    name: String,

} 