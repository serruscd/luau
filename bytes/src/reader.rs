use winnow::{ModalResult, Parser, binary};

use crate::error::error;

pub struct ByteReader<'a> {
    data: &'a mut &'a [u8],
    pub position: usize,
}

impl<'a> ByteReader<'a> {
    pub fn new(data: &'a mut &'a [u8]) -> Self {
        Self { data, position: 0 }
    }

    pub fn u8(&mut self) -> ModalResult<u8> {
        binary::u8.parse_next(self.data)
    }

    pub fn u16(&mut self) -> ModalResult<u16> {
        binary::le_u16.parse_next(self.data)
    }

    pub fn u32(&mut self) -> ModalResult<u32> {
        binary::le_u32.parse_next(self.data)
    }

    pub fn u64(&mut self) -> ModalResult<u64> {
        binary::le_u64.parse_next(self.data)
    }

    pub fn i8(&mut self) -> ModalResult<i8> {
        binary::i8.parse_next(self.data)
    }

    pub fn i16(&mut self) -> ModalResult<i16> {
        binary::le_i16.parse_next(self.data)
    }

    pub fn i32(&mut self) -> ModalResult<i32> {
        binary::le_i32.parse_next(self.data)
    }

    pub fn i64(&mut self) -> ModalResult<i64> {
        binary::le_i64.parse_next(self.data)
    }

    pub fn f32(&mut self) -> ModalResult<f32> {
        binary::le_f32.parse_next(self.data)
    }

    pub fn f64(&mut self) -> ModalResult<f64> {
        binary::le_f64.parse_next(self.data)
    }

    pub fn varint_u32(&mut self) -> ModalResult<u32> {
        let mut result: u32 = 0;
        let mut shift: u32 = 0;

        loop {
            let byte = binary::u8.parse_next(self.data)?;

            if shift == 28 && (byte & 0xF0) != 0 {
                return error("invalid leb128 encoding");
            }

            result |= (byte as u32 & 0x7F) << shift;
            shift += 7;

            if (byte & 0x80) == 0 {
                return Ok(result);
            }

            if shift >= 35 {
                return error("value too big");
            }
        }
    }

    pub fn varint_u64(&mut self) -> ModalResult<u32> {
        let mut result: u32 = 0;
        let mut shift: u32 = 0;

        loop {
            let byte = binary::u8.parse_next(self.data)?;

            if shift == 28 && (byte & 0xF0) != 0 {
                return error("invalid leb128 encoding");
            }

            result |= (byte as u32 & 0x7F) << shift;
            shift += 7;

            if (byte & 0x80) == 0 {
                return Ok(result);
            }

            if shift >= 63 {
                return error("value too big");
            }
        }
    }
}
