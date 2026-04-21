use std::collections::HashMap;

use crate::bytecode::common::bytecode::LuauConstantType;
use crate::bytecode::types::strings::Strings;
use crate::bytes::reader::ByteReader;

#[cfg(feature = "write")]
use crate::bytes::writer::ByteWriter;

#[derive(Clone, Debug)]
#[repr(u8)]
pub enum Constant {
    Nil,
    Boolean(bool),
    Number(f64),
    Vector { x: f32, y: f32, z: f32, w: f32 },
    String(String),
    Import(String, Option<String>, Option<String>),
    Table(Vec<String>),
    ConstantTable(HashMap<String, Constant>),
    Closure(usize),
    Integer(i64),
}

impl Constant {
    pub fn from_reader(
        reader: &mut ByteReader,
        string_table: &Strings,
        constant_table: Vec<Constant>,
    ) -> Option<Self> {
        let constant_type = LuauConstantType::try_from(reader.u8().ok()?).ok()?;
        match constant_type {
            LuauConstantType::LBC_CONSTANT_NIL => return Some(Self::Nil),
            LuauConstantType::LBC_CONSTANT_BOOLEAN => {
                let value = reader.u8().ok()? == 1;
                Some(Self::Boolean(value))
            }
            LuauConstantType::LBC_CONSTANT_NUMBER => {
                let value = reader.f64().ok()?;
                Some(Self::Number(value))
            }
            LuauConstantType::LBC_CONSTANT_VECTOR => {
                let x = reader.f32().ok()?;
                let y = reader.f32().ok()?;
                let z = reader.f32().ok()?;
                let w = reader.f32().ok()?;
                Some(Self::Vector { x, y, z, w })
            }
            LuauConstantType::LBC_CONSTANT_STRING => {
                let index = reader.varint_u32().ok()? as usize;
                Some(Self::String(string_table.get(index)?))
            }
            LuauConstantType::LBC_CONSTANT_IMPORT => {
                let magic = reader.u32().ok()?;
                let size = magic >> 30;

                match size {
                    1 => {
                        let id0 = match constant_table.get((magic >> 20 & 1023) as usize) {
                            Some(Constant::String(s)) => s.clone(),
                            _ => return None,
                        };
                        Some(Self::Import(id0, None, None))
                    }
                    2 => {
                        let id0 = match constant_table.get((magic >> 20 & 1023) as usize) {
                            Some(Constant::String(s)) => s.clone(),
                            _ => return None,
                        };
                        let id1 = match constant_table.get((magic >> 10 & 1023) as usize) {
                            Some(Constant::String(s)) => s.clone(),
                            _ => return None,
                        };
                        Some(Self::Import(id0, Some(id1), None))
                    }
                    3 => {
                        let id0 = match constant_table.get((magic >> 20 & 1023) as usize) {
                            Some(Constant::String(s)) => s.clone(),
                            _ => return None,
                        };
                        let id1 = match constant_table.get((magic >> 10 & 1023) as usize) {
                            Some(Constant::String(s)) => s.clone(),
                            _ => return None,
                        };
                        let id2 = match constant_table.get((magic >> 0 & 1023) as usize) {
                            Some(Constant::String(s)) => s.clone(),
                            _ => return None,
                        };
                        Some(Self::Import(id0, Some(id1), Some(id2)))
                    }
                    _ => return None,
                }
            }
            LuauConstantType::LBC_CONSTANT_TABLE => {
                let new_constant_table_size = reader.varint_u32().ok()?;
                let mut new_constant_table = Vec::with_capacity(new_constant_table_size as usize);

                for _ in 0..new_constant_table_size {
                    match constant_table.get(reader.varint_u32().ok()? as usize) {
                        Some(Constant::String(s)) => new_constant_table.push(s.clone()),
                        _ => return None,
                    }
                }

                Some(Constant::Table(new_constant_table))
            }
            LuauConstantType::LBC_CONSTANT_TABLE_WITH_CONSTANTS => {
                let new_constant_table_size = reader.varint_u32().ok()?;
                let mut new_constant_table_with_constants =
                    HashMap::with_capacity(new_constant_table_size as usize);

                for _ in 0..new_constant_table_size {
                    let key = match constant_table.get(reader.varint_u32().ok()? as usize) {
                        Some(Constant::String(s)) => s.clone(),
                        _ => return None,
                    };
                    let value = match constant_table.get(reader.i32().ok()? as usize) {
                        Some(c) => c.clone(),
                        _ => Constant::Number(0.0f64),
                    };
                    new_constant_table_with_constants.insert(key, value);
                }

                Some(Constant::ConstantTable(new_constant_table_with_constants))
            }
            LuauConstantType::LBC_CONSTANT_CLOSURE => {
                let proto_index = reader.varint_u32().ok()? as usize;
                Some(Constant::Closure(proto_index as usize))
            }
            LuauConstantType::LBC_CONSTANT_INTEGER => {
                let is_negative = reader.u8().ok()? != 0;
                let magnitude = reader.varint_u64().ok()?;
                let value = if is_negative {
                    (!magnitude + 1) as i64
                } else {
                    magnitude as i64
                };
                Some(Constant::Integer(value))
            }
        }
    }

    #[cfg(feature = "write")]
    pub fn to_writer(&self, writer: &mut ByteWriter) {
        match self {
            Constant::Nil => writer.u8(0),
            Constant::Boolean(b) => {
                writer.u8(1);
                writer.u8(*b as u8)
            }
            Constant::Number(n) => {
                writer.u8(2);
                writer.f64(*n);
            }
            Constant::Vector { x, y, z, w } => {
                writer.u8(3);
                writer.f32(*x);
                writer.f32(*y);
                writer.f32(*z);
                writer.f32(*w);
            }
            Constant::String(s) => {
                writer.u8(4);
                writer.raw(s.as_bytes());
            }
            Constant::Import(module, name, alias) => {
                writer.u8(5);
                writer.raw(module.as_bytes());
                if let Some(name) = name {
                    writer.raw(name.as_bytes());
                }
                if let Some(alias) = alias {
                    writer.raw(alias.as_bytes());
                }
            }
            Constant::Table(t) => {
                writer.u8(6);
                writer.varint_u32(t.len() as u32);
                for key in t {
                    writer.raw(key.as_bytes());
                }
            }
            Constant::ConstantTable(t) => {
                writer.u8(7);
                writer.varint_u32(t.len() as u32);
                for (key, value) in t {
                    writer.raw(key.as_bytes());
                    value.to_writer(writer);
                }
            }
            Constant::Closure(i) => {
                writer.u8(8);
                writer.varint_u32(*i as u32);
            }
            Constant::Integer(i) => {
                writer.u8(9);
                writer.varint_u64(*i as u64);
            }
        }
    }
}
