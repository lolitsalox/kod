use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Code {
    pub name: String,
    pub params: Vec<String>,
    pub code: Vec<u8>,
}

impl Code {
    pub fn new(name: String, params: Vec<String>, code: Vec<u8>) -> Self {
        Self { name, params, code }
    }

    pub fn print(&self) {
        println!("name: {}, params: {:?}", self.name, self.params);
        let mut i = 0;
        while i < self.code.len() {
            let opcode = Opcode::try_from(self.read8(&mut i)).unwrap();

            print!("{}: {:?} ", i - 1, opcode);

            match opcode {
                Opcode::JUMP |
                Opcode::POP_JUMP_IF_FALSE |
                Opcode::CALL |
                Opcode::BUILD_TUPLE |
                Opcode::BUILD_LIST |
                Opcode::BUILD_DICT |
                Opcode::LOAD_NAME |
                Opcode::LOAD_METHOD |
                Opcode::LOAD_ATTRIBUTE |
                Opcode::LOAD_ATTRIBUTE_SELF |
                Opcode::STORE_NAME |
                Opcode::STORE_ATTRIBUTE |
                Opcode::LOAD_CONST |
                Opcode::UNPACK_SEQUENCE => { 
                    let index = self.read32(&mut i);
                    print!("{index}");
                },
                Opcode::POP_TOP |
                Opcode::RETURN |
                Opcode::UNARY_ADD |
                Opcode::UNARY_SUB |
                Opcode::UNARY_NOT |
                Opcode::UNARY_BOOL_NOT |
                Opcode::BINARY_ADD |
                Opcode::BINARY_SUB |
                Opcode::BINARY_MUL |
                Opcode::BINARY_DIV |
                Opcode::BINARY_MOD |
                Opcode::BINARY_POW |
                Opcode::BINARY_AND |
                Opcode::BINARY_OR |
                Opcode::BINARY_XOR |
                Opcode::BINARY_LEFT_SHIFT |
                Opcode::BINARY_RIGHT_SHIFT |
                Opcode::BINARY_BOOLEAN_AND |
                Opcode::BINARY_BOOLEAN_OR |
                Opcode::BINARY_BOOLEAN_EQUAL |
                Opcode::BINARY_BOOLEAN_NOT_EQUAL |
                Opcode::BINARY_BOOLEAN_GREATER_THAN |
                Opcode::BINARY_BOOLEAN_GREATER_THAN_OR_EQUAL_TO |
                Opcode::BINARY_BOOLEAN_LESS_THAN |
                Opcode::BINARY_BOOLEAN_LESS_THAN_OR_EQUAL_TO | 
                Opcode::EXTEND_LIST |
                Opcode::SUBSCRIPT => {

                }
            }

            println!();
        }
    }

    pub fn emit(&mut self, other: &[u8]) -> usize {
        self.code.extend_from_slice(other);
        self.code.len() - other.len()
    }

    pub fn emit8(&mut self, byte: u8) -> usize { self.emit(&byte.to_le_bytes()) }

    pub fn emit16(&mut self, word: u16) -> usize { self.emit(&word.to_le_bytes()) }

    pub fn emit32(&mut self, dword: u32) -> usize { self.emit(&dword.to_le_bytes()) }

    pub fn emit64(&mut self, qword: u64) -> usize { self.emit(&qword.to_le_bytes()) }

    pub fn read8(&self, offset: &mut usize) -> u8 {
        let result = self.code[*offset as usize];
        *offset += 1;
        result
    }

    pub fn read32(&self, offset: &mut usize) -> u32 {
        let result = u32::from(self.code[*offset as usize])
            | (u32::from(self.code[*offset as usize + 1]) << 8)
            | (u32::from(self.code[*offset as usize + 2]) << 8*2)
            | (u32::from(self.code[*offset as usize + 3]) << 8*3);

        *offset += 4;
        result
    }

    pub fn read64(&self, offset: &mut usize) -> u64 {
        let result = u64::from(self.code[*offset as usize])
            | (u64::from(self.code[*offset as usize + 1]) << 8)
            | (u64::from(self.code[*offset as usize + 2]) << 8*2)
            | (u64::from(self.code[*offset as usize + 3]) << 8*3)
            | (u64::from(self.code[*offset as usize + 4]) << 8*4)
            | (u64::from(self.code[*offset as usize + 5]) << 8*5)
            | (u64::from(self.code[*offset as usize + 6]) << 8*6)
            | (u64::from(self.code[*offset as usize + 7]) << 8*7);

        *offset += 8;
        result
    }

    pub fn patch32(&mut self, offset: usize, value: u32) {
        self.code[offset] = (value & 0xFF) as u8;
        self.code[offset + 1] = ((value >> 8) & 0xFF) as u8;
        self.code[offset + 2] = ((value >> 16) & 0xFF) as u8;
        self.code[offset + 3] = ((value >> 24) & 0xFF) as u8;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Null,
    Int(i64),
    Float(f64),
    String(String),
    Code(Code),
    Tuple(Vec<Constant>),
}

impl Constant {
    pub fn to_string(&self) {
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub name_pool: Vec<String>,
    pub constant_pool: Vec<Constant>,
    pub entry: Code
}

impl Module {
    pub fn new(name: String) -> Self {
        Self { name, name_pool: vec![], constant_pool: vec![], entry: Code::new(String::new(), vec![], vec![]) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Opcode {
    // LOADs
    LOAD_CONST,      // direct: constant index, stack: none
    LOAD_NAME,       // direct: name index, stack: none
    LOAD_ATTRIBUTE,  // direct: attribute index, stack: this
    LOAD_ATTRIBUTE_SELF,  // direct: attribute index, stack: this
    LOAD_METHOD,     // direct: attribute index, stack: this

    // STOREs
    STORE_NAME,      // direct: name index, stack: object
    STORE_ATTRIBUTE, // direct: attribute index, stack: this, object

    // YEETs
    POP_TOP,

    // OPERATORs
    UNARY_ADD,
    UNARY_SUB,
    UNARY_NOT,
    UNARY_BOOL_NOT,

    BINARY_ADD,
    BINARY_SUB,
    BINARY_MUL,
    BINARY_DIV,
    BINARY_MOD,
    BINARY_POW,

    BINARY_AND,
    BINARY_OR,
    BINARY_XOR,
    BINARY_LEFT_SHIFT,
    BINARY_RIGHT_SHIFT,

    BINARY_BOOLEAN_AND,
    BINARY_BOOLEAN_OR,
    BINARY_BOOLEAN_EQUAL,
    BINARY_BOOLEAN_NOT_EQUAL,
    BINARY_BOOLEAN_GREATER_THAN,
    BINARY_BOOLEAN_GREATER_THAN_OR_EQUAL_TO,
    BINARY_BOOLEAN_LESS_THAN,
    BINARY_BOOLEAN_LESS_THAN_OR_EQUAL_TO,

    // FUNCTIONs
    CALL,                // direct: argument count, stack: object, ...
    RETURN,              // direct: none, stack: object

    // LOOPs
    JUMP,                // direct: relative byte offset, stack: none
    POP_JUMP_IF_FALSE,   // direct: relative byte offset, stack: object

    BUILD_TUPLE,
    BUILD_LIST,
    BUILD_DICT,

    EXTEND_LIST, // direct: none, stack: tuple
    SUBSCRIPT,

    UNPACK_SEQUENCE,
}

impl Opcode {
    pub fn encode(&self) -> u8 {
        *self as u8
    }
}

impl TryFrom<u8> for Opcode {
    type Error = &'static str;

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            // LOADs
            0x00 => Ok(Opcode::LOAD_CONST),
            0x01 => Ok(Opcode::LOAD_NAME),
            0x02 => Ok(Opcode::LOAD_ATTRIBUTE),
            0x03 => Ok(Opcode::LOAD_ATTRIBUTE_SELF),
            0x04 => Ok(Opcode::LOAD_METHOD),

            // STOREs
            0x05 => Ok(Opcode::STORE_NAME),
            0x06 => Ok(Opcode::STORE_ATTRIBUTE),

            // YEETs
            0x07 => Ok(Opcode::POP_TOP),

            // OPERATORs
            0x08 => Ok(Opcode::UNARY_ADD),
            0x09 => Ok(Opcode::UNARY_SUB),
            0x0A => Ok(Opcode::UNARY_NOT),
            0x0B => Ok(Opcode::UNARY_BOOL_NOT),

            0x0C => Ok(Opcode::BINARY_ADD),
            0x0D => Ok(Opcode::BINARY_SUB),
            0x0E => Ok(Opcode::BINARY_MUL),
            0x0F => Ok(Opcode::BINARY_DIV),
            0x10 => Ok(Opcode::BINARY_MOD),
            0x11 => Ok(Opcode::BINARY_POW),

            0x12 => Ok(Opcode::BINARY_AND),
            0x13 => Ok(Opcode::BINARY_OR),
            0x14 => Ok(Opcode::BINARY_XOR),
            0x15 => Ok(Opcode::BINARY_LEFT_SHIFT),
            0x16 => Ok(Opcode::BINARY_RIGHT_SHIFT),

            0x17 => Ok(Opcode::BINARY_BOOLEAN_AND),
            0x18 => Ok(Opcode::BINARY_BOOLEAN_OR),
            0x19 => Ok(Opcode::BINARY_BOOLEAN_EQUAL),
            0x1A => Ok(Opcode::BINARY_BOOLEAN_NOT_EQUAL),
            0x1B => Ok(Opcode::BINARY_BOOLEAN_GREATER_THAN),
            0x1C => Ok(Opcode::BINARY_BOOLEAN_GREATER_THAN_OR_EQUAL_TO),
            0x1D => Ok(Opcode::BINARY_BOOLEAN_LESS_THAN),
            0x1E => Ok(Opcode::BINARY_BOOLEAN_LESS_THAN_OR_EQUAL_TO),

            // FUNCTIONs
            0x1F => Ok(Opcode::CALL),
            0x20 => Ok(Opcode::RETURN),

            // LOOPs
            0x21 => Ok(Opcode::JUMP),
            0x22 => Ok(Opcode::POP_JUMP_IF_FALSE),

            0x23 => Ok(Opcode::BUILD_TUPLE),
            0x24 => Ok(Opcode::BUILD_LIST),
            0x25 => Ok(Opcode::BUILD_DICT),

            0x26 => Ok(Opcode::EXTEND_LIST),
            0x27 => Ok(Opcode::SUBSCRIPT),

            0x28 => Ok(Opcode::UNPACK_SEQUENCE),

            0x80..=0xFF => Err("Unknown opcode"), // Add a catch-all for unknown opcodes

            _ => Err("Invalid opcode"),
        }
    }
}