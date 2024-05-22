#[derive(num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum AuthResult {
    ServerClosed = 1,
    AlreadyLoggedWithId = 2,
    AlreadyOnline = 8
}