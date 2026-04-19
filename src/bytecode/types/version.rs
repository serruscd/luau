use crate::{
    bytecode::common::bytecode::{TYPE_VERSION_MAX, TYPE_VERSION_MIN, VERSION_MAX, VERSION_MIN},
    bytes::reader::ByteReader,
};

pub struct Version {
    pub bytecode: u8,
    pub types: Option<u8>,
}

impl Version {
    pub fn from_reader(reader: &mut ByteReader) -> Option<Self> {
        let bytecode = reader.u8().ok()?;
        let types = reader.u8().ok()?;

        if bytecode < VERSION_MIN || bytecode > VERSION_MAX {
            return None;
        }
        if bytecode >= 4 {
            if types < TYPE_VERSION_MIN || types > TYPE_VERSION_MAX {
                return None;
            } else {
                Some(Self {
                    bytecode,
                    types: Some(types),
                })
            }
        } else {
            Some(Self {
                bytecode,
                types: None,
            })
        }
    }
}
