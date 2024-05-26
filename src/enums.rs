#[derive(num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum AuthResult {
    ServerClosed = 1,
    AlreadyLoggedWithId = 2,
    AlreadyOnline = 8,
}

#[derive(num_enum::TryFromPrimitive)]
#[repr(u16)]
pub enum StatusPoint {
    SpSpeed,
    SpBaseexp,
    SpJobexp,
    SpKarma,
    SpManner,
    SpHp,
    SpMaxhp,
    SpSp, // 0-7
    SpMaxsp,
    SpStatuspoint,
    Sp0a,
    SpBaselevel,
    SpSkillpoint,
    SpStr,
    SpAgi,
    SpVit, // 8-15
    SpInt,
    SpDex,
    SpLuk,
    SpClass,
    SpZeny,
    SpSex,
    SpNextbaseexp,
    SpNextjobexp, // 16-23
    SpWeight,
    SpMaxweight,
    Sp1a,
    Sp1b,
    Sp1c,
    Sp1d,
    Sp1e,
    Sp1f, // 24-31
    SpUstr,
    SpUagi,
    SpUvit,
    SpUint,
    SpUdex,
    SpUluk,
    Sp26,
    Sp27, // 32-39
    Sp28,
    SpAtk1,
    SpAtk2,
    SpMatk1,
    SpMatk2,
    SpDef1,
    SpDef2,
    SpMdef1, // 40-47
    SpMdef2,
    SpHit,
    SpFlee1,
    SpFlee2,
    SpCritical,
    SpAspd,
    Sp36,
    SpJoblevel, // 48-55
    SpUpper,
    SpPartner,
    SpCart,
    SpFame,
    SpUnbreakable,   //56-60
    SpCartinfo = 99, // 99

    // 4TH JOBS
    SpPow = 219,
    SpSta,
    SpWis,
    SpSpl,
    SpCon,
    SpCrt,
    SpPatk,
    SpSmatk,
    SpRes,
    SpMres,
    SpHplus,
    SpCrate,
    SpTraitpoint,
    SpAp,
    SpMaxap,
    SpUpow = 247,
    SpUsta,
    SpUwis,
    SpUspl,
    SpUcon,
    SpUcrt,
}