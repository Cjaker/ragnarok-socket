use crate::input_message::InputMessage;

pub async fn read_pos(data: &mut InputMessage) -> (u16, u16, u8) {
    // pos formula
    let x_byte = data.read_u8();
    let y_byte = data.read_u8();
    let dir_byte = data.read_u8();

    let x = (((x_byte as i16) << 2) | ((y_byte as i16) >> 6)) as u16;
    let y = ((((y_byte & 0x3F) as i16) << 4) | ((dir_byte as i16) >> 4)) as u16;
    let dir = (dir_byte & 0xF) as u8;

    return (x, y, dir)
}
