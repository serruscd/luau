use crate::bytecode::types::{proto::Proto, strings::Strings, version::Version};
use crate::bytes::reader::ByteReader;

#[derive(Debug)]
pub struct LuauChunk {
    pub version: Version,
    pub strings: Strings,
    pub userdata_remapping: Vec<u8>,
    pub protos: Vec<Proto>,
    pub main_proto_id: u32,
}

impl LuauChunk {
    pub fn deserialize(mut reader: ByteReader) -> Option<Self> {
        let version_number = reader.u8().ok()?;

        if version_number == 0 {
            return None;
        }

        let types_version = if version_number >= 4 {
            Some(reader.u8().ok()?)
        } else {
            None
        };

        let version = Version {
            bytecode: version_number,
            types: types_version,
        };

        let string_count = reader.varint_u32().ok()?;
        let mut strings = Strings::new();
        for _ in 0..string_count {
            let str_len = reader.varint_u32().ok()? as usize;
            let str_data = reader.raw(str_len).ok()?;
            strings.add(String::from_utf8_lossy(str_data).into_owned());
        }

        let mut userdata_remapping = Vec::new();
        if let Some(3) = version.types {
            let mut index = reader.u8().ok()?;
            while index != 0 {
                let _name_index = reader.varint_u32().ok()?; // String ref
                userdata_remapping.push(index);
                index = reader.u8().ok()?;
            }
        }

        let proto_count = reader.varint_u32().ok()?;

        let mut protos: Vec<Proto> = Vec::with_capacity(proto_count as usize);

        for i in 0..proto_count {
            let proto = Proto::from_reader(&mut reader, i, &version, &strings)?;
            protos.push(proto);
        }

        let main_proto_id = reader.varint_u32().ok()?;

        Some(Self {
            version,
            strings,
            userdata_remapping,
            protos,
            main_proto_id,
        })
    }
}
