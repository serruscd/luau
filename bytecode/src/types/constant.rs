use bytes::reader::ByteReader;
use std::collections::HashMap;

use crate::common::bytecode::LuauConstantType;

#[derive(Clone, Debug)]
pub enum Constant {
    Nil,
    Boolean(bool),
    Number(f64),
    Vector { x: f32, y: f32, z: f32, w: f32 },
    String(String),
    Import(usize, Option<usize>, Option<usize>),
    Table(Vec<String>),
    ConstantTable(HashMap<String, Constant>),
    Closure(u32),
    Integer(i64),
}

impl Constant {
    pub fn from_reader(
        reader: &mut ByteReader,
        string_table: &Vec<u8>,
        constant_table: &Vec<Constant>,
    ) -> Option<Self> {
        let constant_type = LuauConstantType::try_from(reader.u8().ok()?).ok()?;
        match constant_type {
            LuauConstantType::LBC_CONSTANT_NIL => return Some(Self::Nil),
            LuauConstantType::LBC_CONSTANT_BOOLEAN => {
                let value = reader.u8().ok()? == 1;
                return Some(Self::Boolean(value));
            }
            LuauConstantType::LBC_CONSTANT_NUMBER => {
                let value = reader.f64().ok()?;
                return Some(Self::Number(value));
            }
            LuauConstantType::LBC_CONSTANT_VECTOR => {
                let x = reader.f32().ok()?;
                let y = reader.f32().ok()?;
                let z = reader.f32().ok()?;
                let w = reader.f32().ok()?;
                return Some(Self::Vector { x, y, z, w });
            }
            LuauConstantType::LBC_CONSTANT_STRING => {
                let index = reader.varint_u32().ok()? as usize;
                let value = String::from_utf8_lossy(&string_table[index..]);
                return Some(Self::String(value.into_owned()));
            }
            _ => return Some(Self::Nil),
        }
    }
}
