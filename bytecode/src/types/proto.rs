use crate::types::constant::Constant;

#[derive(Debug, Clone)]
pub struct TypedUpval {
    pub upval_type: u8,
}

#[derive(Debug, Clone)]
pub struct TypedLocal {
    pub local_type: u8,
    pub reg: u8,
    pub start_pc: u32,
    pub size: u32,
}

#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub func_type_info: Vec<u8>,
    pub typed_upvals: Vec<TypedUpval>,
    pub typed_locals: Vec<TypedLocal>,
}

#[derive(Debug, Clone)]
pub struct DebugLocal {
    pub name: u32,
    pub start_pc: u32,
    pub end_pc: u32,
    pub reg: u8,
}

#[derive(Debug, Clone)]
pub struct DebugUpval {
    pub name: u32,
}

#[derive(Debug, Clone)]
pub struct Proto {
    pub id: u32,
    pub max_stack_size: u8,
    pub parameter_count: u8,
    pub upvalue_count: u8,
    pub is_vararg: bool,

    pub flags: Option<u8>,
    pub type_info: Option<TypeInfo>,

    pub code_table: Vec<u32>,
    pub constant_table: Vec<Constant>,
    pub child_protos: Vec<usize>,

    pub debug_line_defined: u32,
    pub debug_name: String,
    pub lines: Option<Vec<i32>>,
    pub debug_locals: Option<Vec<DebugLocal>>,
    pub debug_upvals: Option<Vec<DebugUpval>>,
}
