
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