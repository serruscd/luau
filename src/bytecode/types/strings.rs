use crate::bytes::reader::ByteReader;

#[cfg(feature = "write")]
use crate::bytes::writer::ByteWriter;

#[derive(Debug)]
pub struct Strings {
    pub table: Vec<String>,
}

impl Strings {
    pub fn new() -> Self {
        Self { table: Vec::new() }
    }

    pub fn from_reader(reader: &mut ByteReader) -> Option<Self> {
        let count = reader.varint_u32().ok()?;
        let mut table = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let len = reader.varint_u32().ok()?;
            let bytes = reader.raw(len as usize).ok()?;
            table.push(String::from_utf8_lossy(bytes).into_owned());
        }

        Some(Self { table })
    }

    #[cfg(feature = "write")]
    pub fn to_writer(&self, writer: &mut ByteWriter) {
        writer.varint_u32(self.table.len() as u32);
        for string in &self.table {
            writer.varint_u32(string.len() as u32);
            writer.raw(string.as_bytes());
        }
    }

    pub fn get(&self, index: usize) -> Option<String> {
        self.table.get(index - 1).cloned()
    }

    pub fn add(&mut self, value: String) {
        self.table.push(value);
    }
}
