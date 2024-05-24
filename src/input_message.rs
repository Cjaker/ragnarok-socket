pub struct InputMessage {
    pub data: Vec<u8>,
    pub length: usize,
    pub position: usize
}

impl InputMessage {
    pub fn new(data: Vec<u8>) -> InputMessage {
        let length = data.len();

        InputMessage {
            data,
            length,
            position: 0
        }
    }

    pub fn read_u8(&mut self) -> u8 {
        let value = self.data[self.position];
        self.position += 1;
        value
    }

    pub fn read_u16(&mut self) -> u16 {
        let value = u16::from_le_bytes([self.data[self.position], self.data[self.position + 1]]);
        self.position += 2;
        value
    }

    pub fn read_u32(&mut self) -> u32 {
        let value = u32::from_le_bytes([
            self.data[self.position],
            self.data[self.position + 1],
            self.data[self.position + 2],
            self.data[self.position + 3]
        ]);
        self.position += 4;
        value
    }

    pub fn read_u64(&mut self) -> u64 {
        let value = u64::from_le_bytes([
            self.data[self.position],
            self.data[self.position + 1],
            self.data[self.position + 2],
            self.data[self.position + 3],
            self.data[self.position + 4],
            self.data[self.position + 5],
            self.data[self.position + 6],
            self.data[self.position + 7],
        ]);

        self.position += 8;
        value
    }

    pub fn read_string(&mut self, n: Option<usize>) -> String {
        let mut string = String::new();
        let mut read = 0;
        let size = n.unwrap_or(0);

        loop {
            if size != 0 && read == size {
                break;
            }

            let byte = self.read_u8();
            if size == 0 && byte == 0 {
                break;
            }

            if byte != 0 {
                string.push(byte as char);
            }

            read += 1;
        }
        string
    }

    pub fn is_eof(&self) -> bool {
        self.position >= self.length
    }

    pub fn skip_bytes(&mut self, bytes: usize) {
        self.position += bytes;
    }
}