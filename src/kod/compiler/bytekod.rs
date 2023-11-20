
pub struct Code {
    pub name: String,
    pub params: Vec<String>,
    pub code: Vec<u8>,
}

impl Code {
    pub fn new(name: String, params: Vec<String>, code: Vec<u8>) -> Self {
        Self { name, params, code }
    }

    pub fn to_string(&self) {
        unimplemented!()
    }

    pub fn emit(&mut self, other: &[u8]) {
        self.code.extend_from_slice(other);
    }

    pub fn emit8(&mut self, byte: u8) { self.code.push(byte); }

    pub fn emit16(&mut self, word: u16) { self.emit(&word.to_le_bytes()); }

    pub fn emit32(&mut self, dword: u32) {
        self.emit(&dword.to_le_bytes());
    }

    pub fn emit64(&mut self, qword: u64) {
        self.emit(&qword.to_le_bytes());
    }

    pub fn read8(&self, offset: &mut u8) -> u8 {
        let result = self.code[*offset as usize];
        *offset += 1;
        result
    }

    pub fn read32(&self, offset: &mut u32) -> u32 {
        let result = u32::from(self.code[*offset as usize])
            | (u32::from(self.code[*offset as usize + 1]) << 8)
            | (u32::from(self.code[*offset as usize + 2]) << 8*2)
            | (u32::from(self.code[*offset as usize + 3]) << 8*3);

        *offset += 4;
        result
    }

    pub fn read64(&self, offset: &mut u64) -> u64 {
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

pub enum Constant {
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

pub struct Module {
    pub(crate) name: String,
    pub(crate) name_pool: Vec<String>,
    pub(crate) constant_pool: Vec<Constant>,
    pub(crate) entry: Code
}

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
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
    pub fn to_string(&self) {
        unimplemented!()
    }

    pub fn encode(&self) -> u8 {
        *self as u8
    }
}