use crate::input_message::InputMessage;
use crate::model::move_data::MoveData;

pub fn read_pos(data: &mut InputMessage) -> (u16, u16, u8) {
    // pos formula
    let x_byte = data.read_u8();
    let y_byte = data.read_u8();
    let dir_byte = data.read_u8();

    let x = (((x_byte as i16) << 2) | ((y_byte as i16) >> 6)) as u16;
    let y = ((((y_byte & 0x3F) as i16) << 4) | ((dir_byte as i16) >> 4)) as u16;
    let dir = (dir_byte & 0xF) as u8;

    return (x, y, dir);
}

pub fn read_move_data(data: &mut InputMessage) -> MoveData {
    let a = data.read_u8();
    let b = data.read_u8();
    let c = data.read_u8();
    let d = data.read_u8();
    let e = data.read_u8();
    let f = data.read_u8(); // ?

    let x0 = ((a as u16 & 0xFF) << 2) | ((b as u16 & 0xC0) >> 6);
    let y0 = ((b as u16 & 0x3F) << 4) | ((c as u16 & 0xF0) >> 4);
    let x1 = ((d as u16 & 0xFC) >> 2) | ((c as u16 & 0x0F) << 6);
    let y1 = ((d as u16 & 0x03) << 8) | (e as u16);

    MoveData::new(x0, y0, x1, y1)
}
