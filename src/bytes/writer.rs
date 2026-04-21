pub struct ByteWriter {
    data: Vec<u8>,
}

impl ByteWriter {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn u8(&mut self, value: u8) {
        self.data.push(value);
    }

    pub fn u16(&mut self, value: u16) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    pub fn u32(&mut self, value: u32) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    pub fn u64(&mut self, value: u64) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    pub fn i8(&mut self, value: i8) {
        self.data.push(value as u8);
    }

    pub fn i16(&mut self, value: i16) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    pub fn i32(&mut self, value: i32) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    pub fn i64(&mut self, value: i64) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    pub fn f32(&mut self, value: f32) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    pub fn f64(&mut self, value: f64) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    pub fn varint_u32(&mut self, value: u32) {
        let mut value = value;
        loop {
            let byte = (value & 0x7F) as u8;
            value >>= 7;
            if value == 0 {
                self.data.push(byte);
                return;
            }
            self.data.push(byte | 0x80);
        }
    }

    pub fn varint_u64(&mut self, value: u64) {
        let mut value = value;
        loop {
            let byte = (value & 0x7F) as u8;
            value >>= 7;
            if value == 0 {
                self.data.push(byte);
                return;
            }
            self.data.push(byte | 0x80);
        }
    }

    pub fn raw(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }
}
