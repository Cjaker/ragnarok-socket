pub struct MoveData {
    pub from_x: u16,
    pub from_y: u16,
    pub to_x: u16,
    pub to_y: u16,
}

impl MoveData {
    pub fn new(from_x: u16, from_y: u16, to_x: u16, to_y: u16) -> MoveData {
        MoveData {
            from_x,
            from_y,
            to_x,
            to_y,
        }
    }
}
