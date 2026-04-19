use crate::{
    bytecode::types::{constant::Constant, strings::Strings, version::Version},
    bytes::reader::ByteReader,
};

#[derive(Debug, Clone)]
pub struct TypedUpval {
    pub upval_type: u8,
}

impl TypedUpval {
    pub fn from_reader(reader: &mut ByteReader) -> Option<Self> {
        let upval_type = reader.u8().ok()?;
        Some(Self { upval_type })
    }
}

#[derive(Debug, Clone)]
pub struct TypedLocal {
    pub local_type: u8,
    pub reg: u8,
    pub start_pc: u32,
    pub size: u32,
}

impl TypedLocal {
    pub fn from_reader(reader: &mut ByteReader) -> Option<Self> {
        let local_type = reader.u8().ok()?;
        let reg = reader.u8().ok()?;
        let start_pc = reader.varint_u32().ok()?;
        let size = reader.varint_u32().ok()?;
        Some(Self {
            local_type,
            reg,
            start_pc,
            size,
        })
    }
}

#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub func_type_info: Vec<u8>,
    pub typed_upvals: Vec<TypedUpval>,
    pub typed_locals: Vec<TypedLocal>,
}

impl TypeInfo {
    pub fn from_reader(reader: &mut ByteReader) -> Option<Self> {
        let func_type_info_size = reader.varint_u32().ok()?;
        let typed_upvals_size = reader.varint_u32().ok()?;
        let typed_locals_size = reader.varint_u32().ok()?;

        let func_type_info = reader.raw(func_type_info_size as usize).ok()?.to_vec();
        let mut typed_upvals = Vec::with_capacity(typed_upvals_size as usize);
        let mut typed_locals = Vec::with_capacity(typed_locals_size as usize);

        for _ in 0..typed_upvals_size {
            typed_upvals.push(TypedUpval::from_reader(reader)?);
        }
        for _ in 0..typed_locals_size {
            typed_locals.push(TypedLocal::from_reader(reader)?);
        }

        Some(Self {
            func_type_info,
            typed_upvals,
            typed_locals,
        })
    }
}

#[derive(Debug, Clone)]
pub struct DebugLocal {
    pub name: Option<String>,
    pub start_pc: u32,
    pub end_pc: u32,
    pub reg: u8,
}

#[derive(Debug, Clone)]
pub struct DebugUpval {
    pub name: u32,
}

#[derive(Debug, Clone)]
pub struct Proto<'a> {
    pub id: u32,
    pub max_stack_size: u8,
    pub parameter_count: u8,
    pub upvalue_count: u8,
    pub is_vararg: bool,

    pub flags: Option<u8>,
    pub type_info: Option<TypeInfo>,

    pub code_table: Vec<u32>,
    pub constant_table: Vec<Constant<'a>>,
    pub child_protos: Vec<&'a Proto<'a>>,

    pub debug_line_defined: u32,
    pub debug_name: Option<String>,
    pub line_info: Option<Vec<i32>>,
    pub debug_locals: Option<Vec<DebugLocal>>,
    pub debug_upvals: Option<Vec<DebugUpval>>,
}

impl DebugLocal {
    pub fn from_reader(reader: &mut ByteReader, string_table: &Strings) -> Option<Self> {
        let name_index = reader.varint_u32().ok()?;
        let start_pc = reader.varint_u32().ok()?;
        let end_pc = reader.varint_u32().ok()?;
        let reg = reader.u8().ok()?;
        Some(Self {
            name: string_table.get(name_index as usize),
            start_pc,
            end_pc,
            reg,
        })
    }
}

impl<'a> Proto<'a> {
    pub fn from_reader(
        reader: &mut ByteReader,
        id: u32,
        version: &Version,
        string_table: &Strings,
        constant_table: &'a Vec<Constant<'a>>,
        proto_table: &'a Vec<Proto<'a>>,
    ) -> Option<Self> {
        let max_stack_size = reader.u8().ok()?;
        let parameter_count = reader.u8().ok()?;
        let upvalue_count = reader.u8().ok()?;
        let is_vararg = reader.u8().ok()?;

        let (flags, type_info) = if let Some(type_version) = version.types {
            let flags = reader.u8().ok()?;
            let type_size = reader.varint_u32().ok()?;

            let type_info = if type_size > 0 {
                let type_data = reader.raw(type_size as usize).ok()?;
                let type_reader = ByteReader::new(&mut type_data);
                TypeInfo::from_reader(&mut type_reader)
            } else {
                None
            };

            (Some(flags), type_info)
        } else {
            (None, None)
        };
        
        
    }
}
