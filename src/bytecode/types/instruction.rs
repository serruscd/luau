use crate::bytecode::common::bytecode::LuauOpcode;
use crate::bytes::reader::ByteReader;
use std::convert::TryFrom;

#[cfg(feature = "write")]
use crate::bytes::writer::ByteWriter;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    pub opcode: LuauOpcode,
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: i16,
    pub e: i32,
    pub aux: Option<u32>,
}

impl Instruction {
    /// Decodes a Luau instruction from a direct ByteReader stream.
    pub fn from_reader(reader: &mut ByteReader) -> Option<Self> {
        let word = reader.u32().ok()?;

        let opcode_raw = (word & 0xFF) as u8;
        let opcode = LuauOpcode::try_from(opcode_raw).ok()?;

        let a = ((word >> 8) & 0xFF) as u8;
        let b = ((word >> 16) & 0xFF) as u8;
        let c = ((word >> 24) & 0xFF) as u8;

        // D is a signed 16-bit integer starting at bit 16
        let d = (word >> 16) as i16;

        // E is a signed 24-bit integer starting at bit 8
        let e = (word as i32) >> 8;

        let mut instruction = Self {
            opcode,
            a,
            b,
            c,
            d,
            e,
            aux: None,
        };

        // Certain opcodes are followed by a 4-byte auxiliary value
        if instruction.has_aux() {
            instruction.aux = Some(reader.u32().ok()?);
        }

        Some(instruction)
    }

    /// Creates an Instruction from an already extracted 32-bit word and an optional auxiliary word.
    /// This is useful when parsing from a `Proto`'s `code_table` (which stores raw u32s).
    pub fn from_words(word: u32, aux_word: Option<u32>) -> Option<Self> {
        let opcode_raw = (word & 0xFF) as u8;
        let opcode = LuauOpcode::try_from(opcode_raw).ok()?;

        let a = ((word >> 8) & 0xFF) as u8;
        let b = ((word >> 16) & 0xFF) as u8;
        let c = ((word >> 24) & 0xFF) as u8;

        let d = (word >> 16) as i16;
        let e = (word as i32) >> 8;

        let mut instruction = Self {
            opcode,
            a,
            b,
            c,
            d,
            e,
            aux: None,
        };

        if instruction.has_aux() {
            instruction.aux = aux_word;
        }

        Some(instruction)
    }

    #[cfg(feature = "write")]
    /// Serializes the instruction to the ByteWriter.
    pub fn to_writer(&self, writer: &mut ByteWriter) {
        writer.u32(self.to_word());

        if let Some(aux) = self.aux {
            writer.u32(aux);
        }
    }

    /// Packs the instruction back into a single 32-bit little-endian word.
    /// Note: This relies on `A`, `B`, and `C`. If you manually update `D` or `E`,
    /// use `set_d` and `set_e` respectively to ensure bits remain synchronized.
    pub fn to_word(&self) -> u32 {
        let mut word: u32 = (self.opcode.clone() as u8) as u32;
        word |= (self.a as u32) << 8;
        word |= (self.b as u32) << 16;
        word |= (self.c as u32) << 24;
        word
    }

    /// Helper to cleanly set the `D` argument, which automatically updates `B` and `C` values.
    pub fn set_d(&mut self, value: i16) {
        self.d = value;
        let d_u16 = value as u16;
        self.b = (d_u16 & 0xFF) as u8;
        self.c = ((d_u16 >> 8) & 0xFF) as u8;
    }

    /// Helper to cleanly set the `E` argument, which automatically updates `A`, `B`, and `C` values.
    pub fn set_e(&mut self, value: i32) {
        self.e = value;
        let e_u32 = value as u32;
        self.a = (e_u32 & 0xFF) as u8;
        self.b = ((e_u32 >> 8) & 0xFF) as u8;
        self.c = ((e_u32 >> 16) & 0xFF) as u8;
    }

    /// Determines if an opcode requires an auxiliary (AUX) word based on the Luau specification.
    pub fn has_aux(&self) -> bool {
        match self.opcode {
            LuauOpcode::LOP_GETGLOBAL
            | LuauOpcode::LOP_SETGLOBAL
            | LuauOpcode::LOP_GETIMPORT
            | LuauOpcode::LOP_GETTABLEKS
            | LuauOpcode::LOP_SETTABLEKS
            | LuauOpcode::LOP_NAMECALL
            | LuauOpcode::LOP_NEWTABLE
            | LuauOpcode::LOP_SETLIST
            | LuauOpcode::LOP_FORGLOOP
            | LuauOpcode::LOP_LOADKX
            | LuauOpcode::LOP_FASTCALL2
            | LuauOpcode::LOP_FASTCALL2K
            | LuauOpcode::LOP_FASTCALL3
            | LuauOpcode::LOP_JUMPXEQKNIL
            | LuauOpcode::LOP_JUMPXEQKB
            | LuauOpcode::LOP_JUMPXEQKN
            | LuauOpcode::LOP_JUMPXEQKS
            | LuauOpcode::LOP_GETUDATAKS
            | LuauOpcode::LOP_SETUDATAKS
            | LuauOpcode::LOP_NAMECALLUDATA => true,

            // Equality/jump instructions typically use AUX for the second register check
            LuauOpcode::LOP_JUMPIFEQ
            | LuauOpcode::LOP_JUMPIFLE
            | LuauOpcode::LOP_JUMPIFLT
            | LuauOpcode::LOP_JUMPIFNOTEQ
            | LuauOpcode::LOP_JUMPIFNOTLE
            | LuauOpcode::LOP_JUMPIFNOTLT => true,

            _ => false,
        }
    }
}
