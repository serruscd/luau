use crate::{
    bytecode::types::{
        constant::Constant, instruction::Instruction, strings::Strings, version::Version,
    },
    bytes::reader::ByteReader,
};

#[cfg(feature = "write")]
use crate::bytes::writer::ByteWriter;

#[derive(Debug, Clone)]
pub struct TypedUpval {
    pub upval_type: u8,
}

impl TypedUpval {
    pub fn from_reader(reader: &mut ByteReader) -> Option<Self> {
        let upval_type = reader.u8().ok()?;
        Some(Self { upval_type })
    }

    #[cfg(feature = "write")]
    pub fn to_writer(&self, writer: &mut ByteWriter) {
        writer.u8(self.upval_type);
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

    #[cfg(feature = "write")]
    pub fn to_writer(&self, writer: &mut ByteWriter) {
        writer.u8(self.local_type);
        writer.u8(self.reg);
        writer.varint_u32(self.start_pc);
        writer.varint_u32(self.size);
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

    #[cfg(feature = "write")]
    pub fn to_writer(&self, writer: &mut ByteWriter) {
        writer.varint_u32(self.func_type_info.len() as u32);
        writer.raw(&self.func_type_info);
        writer.varint_u32(self.typed_upvals.len() as u32);
        for upval in &self.typed_upvals {
            upval.to_writer(writer);
        }
        writer.varint_u32(self.typed_locals.len() as u32);
        for local in &self.typed_locals {
            local.to_writer(writer);
        }
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
pub struct Proto {
    pub id: u32,
    pub max_stack_size: u8,
    pub parameter_count: u8,
    pub upvalue_count: u8,
    pub is_vararg: bool,

    pub flags: Option<u8>,
    pub type_info: Option<TypeInfo>,

    pub code_table: Vec<Instruction>,
    pub constant_table: Vec<Constant>,
    pub child_protos: Vec<usize>,

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

impl Proto {
    pub fn from_reader(
        reader: &mut ByteReader,
        id: u32,
        version: &Version,
        string_table: &Strings,
    ) -> Option<Self> {
        let max_stack_size = reader.u8().ok()?;
        let parameter_count = reader.u8().ok()?;
        let upvalue_count = reader.u8().ok()?;
        let is_vararg = reader.u8().ok()?;

        let (flags, type_info) = if let Some(_) = version.types {
            let flags = reader.u8().ok()?;
            let type_size = reader.varint_u32().ok()?;

            let type_info = if type_size > 0 {
                let mut type_data = reader.raw(type_size as usize).ok()?;
                let mut type_reader = ByteReader::new(&mut type_data);
                TypeInfo::from_reader(&mut type_reader)
            } else {
                None
            };

            (Some(flags), type_info)
        } else {
            (None, None)
        };

        let sizecode = reader.varint_u32().ok()?;
        let mut code_table = Vec::with_capacity(sizecode as usize);
        for _ in 0..sizecode {
            code_table.push(Instruction::from_reader(reader)?);
        }

        let sizek = reader.varint_u32().ok()?;
        let mut constant_table = Vec::with_capacity(sizek as usize);

        for _ in 0..sizek {
            constant_table.push(Constant::from_reader(
                reader,
                string_table,
                constant_table.clone(),
            )?);
        }

        let sizep = reader.varint_u32().ok()?;
        let mut child_protos = Vec::with_capacity(sizep as usize);
        for _ in 0..sizep {
            let fid = reader.varint_u32().ok()? as usize;
            child_protos.push(fid as usize);
        }

        let debug_line_defined = reader.varint_u32().ok()?;
        let debug_name_index = reader.varint_u32().ok()? as usize;
        let debug_name = string_table.get(debug_name_index);

        let has_line_info = reader.u8().ok()? != 0;
        let line_info = if has_line_info {
            let linegaplog2 = reader.u8().ok()?;
            let intervals = ((sizecode - 1) >> linegaplog2) + 1;

            let mut line_info_offsets = Vec::with_capacity(sizecode as usize);
            let mut last_offset = 0u8;
            for _ in 0..sizecode {
                last_offset = last_offset.wrapping_add(reader.u8().ok()?);
                line_info_offsets.push(last_offset as i32);
            }

            let mut abs_line_info = Vec::with_capacity(intervals as usize);
            let mut last_line = 0i32;
            for _ in 0..intervals {
                last_line += reader.i32().ok()?;
                abs_line_info.push(last_line);
            }

            Some(abs_line_info)
        } else {
            None
        };

        let has_debug_info = reader.u8().ok()? != 0;
        let (debug_locals, debug_upvals) = if has_debug_info {
            let sizelocvars = reader.varint_u32().ok()?;
            let mut locals = Vec::with_capacity(sizelocvars as usize);
            for _ in 0..sizelocvars {
                locals.push(DebugLocal::from_reader(reader, string_table)?);
            }

            let sizeupvalues = reader.varint_u32().ok()?;
            let mut upvals = Vec::with_capacity(sizeupvalues as usize);
            for _ in 0..sizeupvalues {
                upvals.push(DebugUpval {
                    name: reader.varint_u32().ok()?,
                });
            }

            (Some(locals), Some(upvals))
        } else {
            (None, None)
        };

        Some(Self {
            id,
            max_stack_size,
            parameter_count,
            upvalue_count,
            is_vararg: is_vararg != 0,
            flags,
            type_info,
            code_table,
            constant_table,
            child_protos,
            debug_line_defined,
            debug_name,
            line_info,
            debug_locals,
            debug_upvals,
        })
    }

    #[cfg(feature = "write")]
    pub fn to_writer(&self, writer: &mut ByteWriter) {
        writer.varint_u32(self.id);
        writer.u8(self.max_stack_size);
        writer.u8(self.parameter_count);
        writer.u8(self.upvalue_count);
        writer.u8(self.is_vararg as u8);

        if let Some(flags) = self.flags {
            writer.u8(flags);
        } else {
            writer.u8(0);
        }

        if let Some(type_info) = &self.type_info {
            writer.u8(1);
            type_info.to_writer(writer);
        } else {
            writer.u8(0);
        }

        writer.varint_u32(self.code_table.len() as u32);
        for code in &self.code_table {
            writer.varint_u32(*code);
        }

        writer.varint_u32(self.constant_table.len() as u32);
        for constant in &self.constant_table {
            constant.to_writer(writer);
        }

        writer.varint_u32(self.child_protos.len() as u32);
        for child in &self.child_protos {
            writer.varint_u32(*child as u32);
        }

        writer.varint_u32(self.debug_line_defined);
        if let Some(name) = &self.debug_name {
            writer.u8(1);
            writer.varint_u32(name.len() as u32);
            writer.raw(name.as_bytes());
        } else {
            writer.u8(0);
        }

        if let Some(line_info) = &self.line_info {
            writer.u8(1);
            writer.varint_u32(line_info.len() as u32);
            for line in line_info {
                writer.i32(*line);
            }
        } else {
            writer.u8(0);
        }

        if let Some(locals) = &self.debug_locals {
            writer.u8(1);
            writer.varint_u32(locals.len() as u32);
            for local in locals {
                local.to_writer(writer);
            }
        } else {
            writer.u8(0);
        }

        if let Some(upvals) = &self.debug_upvals {
            writer.u8(1);
            writer.varint_u32(upvals.len() as u32);
            for upval in upvals {
                upval.to_writer(writer);
            }
        } else {
            writer.u8(0);
        }
    }
}
