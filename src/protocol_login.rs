pub mod client {
    pub static UDPCLHASH: u16 = 0x0204;
    pub static REQAUTH: u16 = 0x0064;
}

#[derive(num_enum::TryFromPrimitive)]
#[repr(u16)]
pub enum LoginServer {
    AuthOk = 0x0AC4,
    AuthResult = 0x0081
}