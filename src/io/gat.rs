use std::{fs::File, io::{Cursor, Read, Seek, SeekFrom}};

pub struct GatData {
    magic: [u8; 6],
    width: u32,
    height: u32,
    blocks: Vec<GatBlock>,
}

pub struct GatBlock {
    upper_left_height: f32,
    upper_right_height: f32,
    lower_left_height: f32,
    lower_right_height: f32,
    r#type: u8,
    unknown: [u8; 3],
}

impl GatData {
    pub fn new() -> Self {
        GatData {
            magic: [0; 6],
            width: 0,
            height: 0,
            blocks: Vec::new(),
        }
    }

    pub fn parse(file_name: &str) -> GatData {
        let mut gat_data = GatData::new();
        let mut file = File::open(file_name).expect("file not found");
        let mut buffer = Vec::new();
        let mut tmp_buffer = [0u8; 32];

        // read all file buffer until end
        file.read_to_end(&mut buffer).expect("failed to read file");

        let mut cursor = Cursor::new(buffer);

        // skip gat header
        cursor.seek(SeekFrom::Start(6)).expect("failed to seek");

        cursor.read(&mut tmp_buffer[0..4]).expect("failed to read width");
        let width = u32::from_le_bytes(tmp_buffer[0..4].try_into().expect("failed to convert width"));
        cursor.read(&mut tmp_buffer[0..4]).expect("failed to read width");
        let height = u32::from_le_bytes(tmp_buffer[0..4].try_into().expect("failed to convert height"));
        println!("width: {}, height: {}", width, height);

        for _ in 0..width {
            for _ in 0..height {
                cursor.read(&mut tmp_buffer[0..4]).expect("failed to read upper_left_height");
                let upper_left_height = f32::from_le_bytes(tmp_buffer[0..4].try_into().expect("failed to convert upper_left_height"));
                cursor.read(&mut tmp_buffer[0..4]).expect("failed to read upper_right_height");
                let upper_right_height = f32::from_le_bytes(tmp_buffer[0..4].try_into().expect("failed to convert upper_right_height"));
                cursor.read(&mut tmp_buffer[0..4]).expect("failed to read lower_left_height");
                let lower_left_height = f32::from_le_bytes(tmp_buffer[0..4].try_into().expect("failed to convert lower_left_height"));
                cursor.read(&mut tmp_buffer[0..4]).expect("failed to read lower_right_height");
                let lower_right_height = f32::from_le_bytes(tmp_buffer[0..4].try_into().expect("failed to convert lower_right_height"));
                cursor.read(&mut tmp_buffer[0..1]).expect("failed to read type");
                let r#type = u8::from_le_bytes(tmp_buffer[0..1].try_into().expect("failed to convert type"));
                cursor.read(&mut tmp_buffer[0..3]).expect("failed to read unknown");
                let unknown = tmp_buffer[0..3].try_into().expect("failed to convert unknown");

                gat_data.blocks.push(GatBlock {
                    upper_left_height,
                    upper_right_height,
                    lower_left_height,
                    lower_right_height,
                    r#type,
                    unknown,
                });
            }
        }

        gat_data
    }
}