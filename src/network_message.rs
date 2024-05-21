use core::panic;
use std::mem;

pub struct NetworkMessage {
    pub length: u16,
    pub position: u16,
    pub buffer: [u8; 1024],
}

impl NetworkMessage {
    pub fn new() -> NetworkMessage {
        NetworkMessage {
            length: 0,
            position: 0,
            buffer: ([0; 1024]),
        }
    }
    pub fn add<T: Into<u64>>(&mut self, value: T) {
        let size = mem::size_of_val(&value);
        let val = value.into() as u64;

        if self.position + size as u16 > self.buffer.len() as u16 {
            panic!("Buffer overflow!");
        }

        let start_pos = self.position as usize;
        match size {
            1 => self.buffer[start_pos..(start_pos + size as usize)]
                .copy_from_slice(&(val as u8).to_le_bytes()),
            2 => self.buffer[start_pos..(start_pos + size as usize)]
                .copy_from_slice(&(val as u16).to_le_bytes()),
            4 => self.buffer[start_pos..(start_pos + size as usize)]
                .copy_from_slice(&(val as u32).to_le_bytes()),
            8 => self.buffer[start_pos..(start_pos + size as usize)]
                .copy_from_slice(&val.to_le_bytes()),
            _ => panic!("Unsupported size"),
        }

        self.position += size as u16;
        self.length += size as u16;
    }
    pub fn add_be<T: Into<u64>>(&mut self, value: T) {
        let size = mem::size_of_val(&value);
        let val = value.into() as u64;

        if self.position + size as u16 > self.buffer.len() as u16 {
            panic!("Buffer overflow!");
        }

        let start_pos = self.position as usize;
        match size {
            1 => self.buffer[start_pos..(start_pos + size as usize)]
                .copy_from_slice(&(val as u8).to_be_bytes()),
            2 => self.buffer[start_pos..(start_pos + size as usize)]
                .copy_from_slice(&(val as u16).to_be_bytes()),
            4 => self.buffer[start_pos..(start_pos + size as usize)]
                .copy_from_slice(&(val as u32).to_be_bytes()),
            8 => self.buffer[start_pos..(start_pos + size as usize)]
                .copy_from_slice(&val.to_be_bytes()),
            _ => panic!("Unsupported size"),
        }

        self.position += size as u16;
        self.length += size as u16;
    }
    pub fn skip_bytes(&mut self, count: usize) {
        self.position += count as u16;
        self.length += count as u16;
    }
    pub fn add_string(&mut self, string: &str) {
        for x in string.bytes() {
            self.add(x);
        }
        // add null-terminator
        self.add(0 as u8);
    }
}
