#[derive(Clone, Debug)]
pub struct TypeDescriptor {
    pub name: String,
    pub kind: TypeKind,
}

#[derive(Clone, Debug)]
pub struct TypeField {
    pub fieldname: String,
    pub kind: TypeDescriptor,
}

#[derive(Clone, Debug)]
pub enum TypeKind {
    Bits(usize),
    Signed(usize),
    Enum(Vec<String>),
    Composite(Vec<Box<TypeField>>),
}
