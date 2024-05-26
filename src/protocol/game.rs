use std::collections::HashMap;

use num_enum::TryFromPrimitive;
use rand::Rng;
use tokio::{io::AsyncReadExt, net::TcpStream};

use crate::{
    client::network::write_message, enums::StatusPoint, input_message::InputMessage,
    network_message::NetworkMessage, protocol::helper::read_pos, r#const::PACKET_HEADER_LEN,
};

#[derive(TryFromPrimitive)]
#[repr(u16)]
pub enum GameClient {
    ConnectMapServer = 0x0436,
    RequestAction = 0x0437,
    EffectsOption = 0x021D,
    AckMap = 0x007D,
    ClientTick = 0x0360,
    ChangeDir = 0x0361,
    ChatMessage = 0x00F3, // global message
}

#[derive(TryFromPrimitive)]
#[repr(u16)]
pub enum GameServer {
    MapBlockList = 0x0283,
    InventoryExpansionInfo = 0x0B18,
    NotifyChangeStatus = 0x02CE,
    AuthOk = 0x02EB,
    DisplayMessage = 0x008E,
    ChangeMap = 0x0091,
    ParameterChange = 0x00B0,
    CoupleStatus = 0x0141,
    AtkRange = 0x013A,
    MailUnread = 0x09E7,
    QuestsStateList = 0x09F8,
    SingleAchievementData = 0x0A24,
    AllAchievementsData = 0x0A23,
    WeightLimit = 0x0ADE,
    SpriteChange = 0x01D7,
    InventoryStart = 0x0B08,
    InventoryType = 0x0B39,
    InventoryEnd = 0x0B0B,
    EquipSwitchList = 0x0A9B,
    MapProperty = 0x099B,
    UnitIdle = 0x09FF,         // unit idle info!
    ScreenActiveEFST = 0x0984, // Notifies the client when a player enters the screen with an active EFST.
    SkillTree = 0x010F,
    ShortcutsKeyList = 0x0B20,
    LongParameterChange = 0x0ACB,
    CharacterStatus = 0x00BD,
    UpdateStatus = 0x00BE, // SP_U<STAT> are used to update the amount of points necessary to increase that stat
    PartyInvitationState = 0x02C9,
    EquipWindowOpen = 0x02DA,
    ConfigurationChange = 0x02D9,
    UnitChangedDir = 0x009C,
}

pub static mut GAME_PACKETS_LEN: Option<HashMap<u16, u16>> = None;

pub async fn game_connect_map_server(
    stream: &mut TcpStream,
    acc_id: u32,
    char_id: u32,
    login_id: u32,
    client_tick: u32,
    sex: u8,
) {
    let mut network_message = NetworkMessage::new();
    network_message.add(GameClient::ConnectMapServer as u16);
    network_message.add(acc_id);
    network_message.add(char_id);
    network_message.add(login_id);
    network_message.add(client_tick as u64);
    network_message.add(sex);

    write_message(stream, &network_message).await;
}

pub async fn game_request_action(stream: &mut TcpStream, target_id: u32, action_type: u8) {
    let mut network_message = NetworkMessage::new();
    network_message.add(GameClient::RequestAction as u16);
    network_message.add(target_id);
    network_message.add(action_type);

    write_message(stream, &network_message).await;
}

pub async fn game_request_effects_option(stream: &mut TcpStream, effects_option: u32) {
    let mut network_message = NetworkMessage::new();
    network_message.add(GameClient::EffectsOption as u16);
    network_message.add(effects_option);

    write_message(stream, &network_message).await;
}

pub async fn game_request_ack_map(stream: &mut TcpStream) {
    let mut network_message = NetworkMessage::new();
    network_message.add(GameClient::AckMap as u16);

    write_message(stream, &network_message).await;
}

pub async fn game_request_client_tick(stream: &mut TcpStream, tick: u32) {
    let mut network_message = NetworkMessage::new();
    network_message.add(GameClient::ClientTick as u16);
    network_message.add(tick);

    write_message(stream, &network_message).await;
}

pub async fn game_request_change_dir(stream: &mut TcpStream, head_dir: u16, dir: u8) {
    let mut network_message = NetworkMessage::new();
    network_message.add(GameClient::ChangeDir as u16);
    network_message.add(head_dir);
    network_message.add(dir);

    write_message(stream, &network_message).await;
}

pub async fn game_request_chat_message(stream: &mut TcpStream, message: &str) {
    let mut network_message = NetworkMessage::new();
    network_message.add(GameClient::ChatMessage as u16);
    network_message.add(0 as u16);

    let message: String = format!("{} : {}", "Testando", message);
    network_message.add_string(&message);

    let packet_len = network_message.length as u16;
    // write packet_len into 2 bytes LE
    network_message.buffer[2] = packet_len as u8;
    network_message.buffer[3] = (packet_len >> 8) as u8;

    write_message(stream, &network_message).await;
}

pub async fn game_map_block_list(data: &mut InputMessage) {
    let acc_id = data.read_u32();
}

pub async fn game_inventory_expansion_info(data: &mut InputMessage) {
    let expansion_size = data.read_u16();
}

pub async fn game_notify_change_status(data: &mut InputMessage) {
    let notify_type = data.read_u32();
    let message_id = data.read_u32();
}

pub async fn game_auth_ok(data: &mut InputMessage) {
    let client_tick = data.read_u32();
    let pos = read_pos(data).await;

    // print x, y, dir
    println!("x: {}, y: {}, dir: {}", pos.0, pos.1, pos.2);

    let _unk1 = data.read_u8();
    let _unk2 = data.read_u8();
    let font = data.read_u16();
}

pub async fn game_display_message(data: &mut InputMessage) {
    let remaining_bytes = data.length - data.position;
    let message = data.read_string(Some(remaining_bytes));

    println!("Message: {}", message);
}

pub async fn game_change_map(data: &mut InputMessage) {
    let map_name = data.read_string(Some(16));
    let x = data.read_u16();
    let y = data.read_u16();
}

pub async fn game_param_change(data: &mut InputMessage) {
    let status_type =
        StatusPoint::try_from_primitive(data.read_u16()).expect("invalid status type");

    match status_type {
        StatusPoint::SpWeight => {
            let value = data.read_u32();
        }
        StatusPoint::SpMaxweight => {
            let value = data.read_u32();
        }
        StatusPoint::SpSpeed => {
            let value = data.read_u32();
        }
        StatusPoint::SpBaselevel => {
            let value = data.read_u32();
        }
        StatusPoint::SpJoblevel => {
            let value = data.read_u32();
        }
        StatusPoint::SpKarma => {
            let value = data.read_u32();
        }
        StatusPoint::SpManner => {
            let value = data.read_u32();
        }
        StatusPoint::SpStatuspoint => {
            let value = data.read_u32();
        }
        StatusPoint::SpSkillpoint => {
            let value = data.read_u32();
        }
        StatusPoint::SpHit => {
            let value = data.read_u32();
        }
        StatusPoint::SpFlee1 => {
            let value = data.read_u32();
        }
        StatusPoint::SpFlee2 => {
            let value = data.read_u32();
        }
        StatusPoint::SpMaxhp => {
            let value = data.read_u32();
        }
        StatusPoint::SpMaxsp => {
            let value = data.read_u32();
        }
        StatusPoint::SpHp => {
            // On officials the HP never go below 1, even if you die [Lemongrass]
            // On officials the HP Novice class never go below 50%, even if you die [Napster]
            let value = data.read_u32();
        }
        StatusPoint::SpSp => {
            let value = data.read_u32();
        }
        StatusPoint::SpAspd => {
            let value = data.read_u32();
        }
        StatusPoint::SpAtk1 => {
            let value = data.read_u32();
        }
        StatusPoint::SpDef1 => {
            let value = data.read_u32();
        }
        StatusPoint::SpMdef1 => {
            let value = data.read_u32();
        }
        StatusPoint::SpAtk2 => {
            let value = data.read_u32();
        }
        StatusPoint::SpDef2 => {
            let value = data.read_u32();
        }
        StatusPoint::SpMdef2 => {
            let value = data.read_u32();
        }
        StatusPoint::SpCritical => {
            let value = data.read_u32();
        }
        StatusPoint::SpMatk1 => {
            let value = data.read_u32();
        }
        StatusPoint::SpMatk2 => {
            let value = data.read_u32();
        }
        _ => {
            // Handle default case if needed
        }
    }
}

pub async fn game_couple_status(data: &mut InputMessage) {
    let status_type =
        StatusPoint::try_from_primitive(data.read_u32() as u16).expect("invalid status type");

    match status_type {
        StatusPoint::SpStr
        | StatusPoint::SpAgi
        | StatusPoint::SpVit
        | StatusPoint::SpInt
        | StatusPoint::SpDex
        | StatusPoint::SpLuk
        | StatusPoint::SpPow
        | StatusPoint::SpSta
        | StatusPoint::SpWis
        | StatusPoint::SpSpl
        | StatusPoint::SpCon
        | StatusPoint::SpCrt => {
            let mut value = data.read_u32();
            let mut plus_value = data.read_u32();
        }
        _ => {
            // Handle default case if needed
        }
    }
}

pub async fn game_atk_range(data: &mut InputMessage) {
    let atk_range = data.read_u16();
}

pub async fn game_mail_unread(data: &mut InputMessage) {
    let is_unread = data.read_u8() == 1;
}

pub async fn game_quests_state_list(data: &mut InputMessage) {
    let quests_size = data.read_u32();
    for _ in 0..quests_size {
        let quest_id = data.read_u32();
        let state = data.read_u8();
        let time = data.read_u32();
        let time2 = data.read_u32();
        let objectives_size = data.read_u16();

        for _ in 0..objectives_size {
            let objective_id = data.read_u32();
            let race = data.read_u16();

            let mob_id = data.read_u32();
            let min_level = data.read_u16();
            let max_level = data.read_u16();
            let count = data.read_u16();
            let count_total = data.read_u16();
            let name = data.read_string(Some(24));
        }
    }
}

pub async fn game_single_achievement_data(data: &mut InputMessage) {
    let total_score = data.read_u32();
    let level = data.read_u16();
    let exp = data.read_u32();
    let exp_tnl = data.read_u32();
    let achievement_id = data.read_u32();
    let is_complete = data.read_u8() == 1;

    let max_achievement_objectives = 10;
    for _ in 0..max_achievement_objectives {
        let pre_req_count = data.read_u32();
    }

    let completed_epoch_time = data.read_u32();
    let is_rewarded = data.read_u8() == 1;
}

pub async fn game_all_achievements_data(data: &mut InputMessage) {
    let count = data.read_u32();
    let total_score = data.read_u32();
    let level = data.read_u16();
    let exp = data.read_u32();
    let exp_tnl = data.read_u32();

    for _ in 0..count {
        let achievement_id = data.read_u32();
        let is_complete = data.read_u8() == 1;

        let max_achievement_objectives = 10;
        for _ in 0..max_achievement_objectives {
            let pre_req_count = data.read_u32();
        }

        let completed_epoch_time = data.read_u32();
        let is_rewarded = data.read_u8() == 1;
    }
}

pub async fn game_weight_limit(data: &mut InputMessage) {
    let weight_percent = data.read_u32();
}

pub async fn game_sprite_change(data: &mut InputMessage) {
    let acc_id = data.read_u32();
    let sprite_type = data.read_u8();
    let val = data.read_u32();
    let val2 = data.read_u32();
}

pub async fn game_inventory_start(data: &mut InputMessage) {
    let inventory_type = data.read_u8();
    let remaining_bytes = data.length - data.position;
    let name = data.read_string(Some(remaining_bytes));
}

pub async fn game_inventory_equip_item(data: &mut InputMessage) {
    let inventory_type = data.read_u8();

    loop {
        if data.is_eof() {
            println!("inventory equip items done! EOF");
            break;
        }

        let index = data.read_u16();
        let it_id = data.read_u32();
        let type_ = data.read_u8();
        let location = data.read_u32();
        let wear_state = data.read_u32();

        //equip_slot_info_struct
        let mut cards = Vec::new();
        for _ in 0..4 {
            let card = data.read_u32();
            cards.push(card);
        }

        let hire_expire_date = data.read_u32();
        let bind_on_equip_type = data.read_u16();
        let w_item_sprite_number = data.read_u16();

        // item_option_struct
        let max_options_count = 5;
        let option_count = data.read_u8();
        for _ in 0..max_options_count {
            let index = data.read_u16();
            let value = data.read_u16();
            let param = data.read_u8();
        }

        let refining_level = data.read_u8();
        let enchant_grade = data.read_u8();

        let flag = data.read_u8(); // 1 bit = IsIdentified | 2nd bit = IsDamaged | 3nd bit = PlaceETCTab | 4+ bits = SpareBits
    }
}

pub async fn game_inventory_end(data: &mut InputMessage) {
    let inventory_type = data.read_u8();
    let flag = data.read_u8();
}

pub async fn game_equip_switch_list(data: &mut InputMessage) {
    loop {
        if data.is_eof() {
            println!("equip switch list done! EOF");
            break;
        }

        let index = data.read_u16();
        let position = data.read_u32();
    }
}

pub async fn game_map_property(data: &mut InputMessage) {
    let property = data.read_u16(); // map_property
    let flags = data.read_u32();

    // flags example below
    // WBUFL(buf,4) = ((mapdata->flag[MF_PVP] || (sd && sd->duel_group > 0))<<0)| // PARTY - Show attack cursor on non-party members (PvP)
    // 	((mapdata->flag[MF_BATTLEGROUND] || mapdata_flag_gvg2(mapdata))<<1)|// GUILD - Show attack cursor on non-guild members (GvG)
    // 	((mapdata->flag[MF_BATTLEGROUND] || mapdata_flag_gvg2(mapdata))<<2)|// SIEGE - Show emblem over characters heads when in GvG (WoE castle)
    // 	((mapdata->flag[MF_NOMINEEFFECT] || mapdata_flag_gvg2(mapdata))<<3)| // USE_SIMPLE_EFFECT - Automatically enable /mineffect
    // 	((mapdata->flag[MF_NOLOCKON] || mapdata_flag_vs(mapdata) || (sd && sd->duel_group > 0))<<4)| // DISABLE_LOCKON - Only allow attacks on other players with shift key or /ns active
    // 	((mapdata->flag[MF_PVP])<<5)| // COUNT_PK - Show the PvP counter
    // 	((mapdata->flag[MF_PARTYLOCK])<<6)| // NO_PARTY_FORMATION - Prevents party creation/modification (Might be used for instance dungeons)
    // 	((mapdata->flag[MF_BATTLEGROUND])<<7)| // BATTLEFIELD - Unknown (Does something for battlegrounds areas)
    // 	((mapdata->flag[MF_NOCOSTUME])<<8)| // DISABLE_COSTUMEITEM - Disable costume sprites
    // 	((!mapdata->flag[MF_NOUSECART])<<9)| // USECART - Allow opening cart inventory (Well force it to always allow it)
    // 	((!mapdata->flag[MF_NOSUNMOONSTARMIRACLE])<<10); // SUNMOONSTAR_MIRACLE - Allows Star Gladiator's Miracle to activate
    // 	//(1<<11); // Unused bits. 1 - 10 is 0x1 length and 11 is 0x15 length. May be used for future settings.
}

pub async fn game_unit_idle(data: &mut InputMessage) {
    let object_type = data.read_u8();
    let aid = data.read_u32();
    let gid = data.read_u32();
    let speed = data.read_u16();
    let body_state = data.read_u16();
    let health_state = data.read_u16();
    let effect_state = data.read_u32();
    let job = data.read_u16();
    let head = data.read_u16();
    let weapon = data.read_u32();
    let shield = data.read_u32();
    let accessory = data.read_u16();
    let accessory2 = data.read_u16();
    let accessory3 = data.read_u16();
    let head_palette = data.read_u16();
    let body_palette = data.read_u16();
    let head_dir = data.read_u16();
    let robe = data.read_u16();
    let guid = data.read_u32();
    let g_emblem_ver = data.read_u16();
    let honor = data.read_u16();
    let virtue = data.read_u32();
    let is_pk_mode_on = data.read_u8() == 1;
    let sex = data.read_u8();
    let pos_dir = data.read_bytes(3);
    let x_size = data.read_u8();
    let y_size = data.read_u8();
    let state = data.read_u8();
    let clevel = data.read_u16();
    let font = data.read_u16();
    let max_hp = data.read_u32();
    let hp = data.read_u32();
    let is_boss = data.read_u8();
    let body = data.read_u16();
    let name = data.read_string(Some(24));
}

pub async fn game_screen_active_esft(data: &mut InputMessage) {
    let index = data.read_u32();
    let status_type = data.read_u16();
    let remain_msec = data.read_u32();
    let unk = data.read_u32();
    let unk1 = data.read_u32();
    let unk2 = data.read_u32();
    let unk3 = data.read_u32();
}

pub async fn game_skill_tree(data: &mut InputMessage) {
    loop {
        if data.is_eof() {
            println!("skill tree done! EOF");
            break;
        }

        let id = data.read_u16();
        let skill_info = data.read_u32();
        let level = data.read_u16();
        let sp = data.read_u16();
        let range = data.read_u16();
        let name = data.read_string(Some(24));
        let is_max = data.read_u8() == 1;
    }
}

pub async fn game_shortcuts_key_list(data: &mut InputMessage) {
    let rotate = data.read_u8();
    let tab = data.read_u16();

    for x in 0..38 {
        //hotkey_data struct
        let is_skill = data.read_u8() == 1; // 0: Item, 1:Skill

        let it_skill_id = data.read_u32();
        let it_count_skill_lv = data.read_u16();
    }
}

pub async fn game_long_parameter_change(data: &mut InputMessage) {
    let parameter_type = data.read_u16();
    let value = data.read_u64();
}

pub async fn game_character_status(data: &mut InputMessage) {
    let status_point = data.read_u16();
    let r#str = data.read_u8();
    let str_needed_sp = data.read_u8();
    let agi = data.read_u8();
    let agi_needed_sp = data.read_u8();
    let vit = data.read_u8();
    let vit_needed_sp = data.read_u8();
    let int = data.read_u8();
    let int_needed_sp = data.read_u8();
    let dex = data.read_u8();
    let dex_needed_sp = data.read_u8();
    let luk = data.read_u8();
    let luk_needed_sp = data.read_u8();
    let left_side_atk = data.read_u16();
    let right_side_atk = data.read_u16();
    let right_side_matk = data.read_u16();
    let left_side_matk = data.read_u16();
    let left_side_def = data.read_u16();
    let right_side_def = data.read_u16();
    let left_side_mdef = data.read_u16();
    let mdef2 = data.read_u16();
    let hit = data.read_u16();
    let flee = data.read_u16();
    let flee2 = data.read_u16();
    let cri = data.read_u16();
    let aspd = data.read_u16();
    let plus_aspd = data.read_u16(); // always 0 (plusASPD)
}

// SP_U<STAT> are used to update the amount of points necessary to increase that stat
pub async fn game_update_status(data: &mut InputMessage) {
    let status_type =
        StatusPoint::try_from_primitive(data.read_u16()).expect("invalid status type");
    match status_type {
        StatusPoint::SpUstr
        | StatusPoint::SpUagi
        | StatusPoint::SpUvit
        | StatusPoint::SpUint
        | StatusPoint::SpUdex
        | StatusPoint::SpUluk
        | StatusPoint::SpUpow
        | StatusPoint::SpUsta
        | StatusPoint::SpUwis
        | StatusPoint::SpUspl
        | StatusPoint::SpUcon
        | StatusPoint::SpUcrt => {
            let value = data.read_u8();
        }
        _ => {
            // Handle default case if needed
            panic!("invalid update status type! type: {}", status_type as u16);
        }
    }
}

pub async fn game_party_invitation_state(data: &mut InputMessage) {
    let state = data.read_u8(); // flags 0 = allow party invites | 1 = auto-deny party invites
}

pub async fn game_equip_window_open(data: &mut InputMessage) {
    let open_equip_window = data.read_u8() == 1;
}

pub async fn game_configuration_change(data: &mut InputMessage) {
    let config_type = data.read_u32();
    let enabled = data.read_u32() == 1;
}

pub async fn game_unit_changed_dir(data: &mut InputMessage) {
    let unit_id = data.read_u32();
    let head_dir = data.read_u16();
    let dir = data.read_u8();
}

pub async fn game_packet_handler(
    stream: &mut TcpStream,
    packet_id: u16,
    data: &mut InputMessage,
) -> bool {
    println!("Packet ID: {:x}", packet_id);
    let packet_id = GameServer::try_from(packet_id)
        .expect(format!("missing packet id {:x}", packet_id).as_str());

    match packet_id {
        GameServer::MapBlockList => {
            game_map_block_list(data).await;
        }
        GameServer::InventoryExpansionInfo => {
            game_inventory_expansion_info(data).await;
        }
        GameServer::NotifyChangeStatus => {
            game_notify_change_status(data).await;
        }
        GameServer::AuthOk => {
            game_auth_ok(data).await;
        }
        GameServer::DisplayMessage => {
            game_display_message(data).await;
        }
        GameServer::ChangeMap => {
            game_change_map(data).await;
        }
        GameServer::ParameterChange => {
            game_param_change(data).await;
        }
        GameServer::CoupleStatus => {
            game_couple_status(data).await;
        }
        GameServer::AtkRange => {
            game_atk_range(data).await;
        }
        GameServer::MailUnread => {
            game_mail_unread(data).await;
        }
        GameServer::QuestsStateList => {
            game_quests_state_list(data).await;
        }
        GameServer::SingleAchievementData => {
            game_single_achievement_data(data).await;
        }
        GameServer::AllAchievementsData => {
            game_all_achievements_data(data).await;
        }
        GameServer::WeightLimit => {
            game_weight_limit(data).await;
            // after this packet, it seems we can start sending our information?
            game_request_effects_option(stream, 0).await;
            // tell to server we are all map data ready
            game_request_ack_map(stream).await;
        }
        GameServer::SpriteChange => {
            game_sprite_change(data).await;
        }
        GameServer::InventoryStart => {
            game_inventory_start(data).await;
        }
        GameServer::InventoryType => {
            game_inventory_equip_item(data).await;
        }
        GameServer::InventoryEnd => {
            game_inventory_end(data).await;
        }
        GameServer::EquipSwitchList => {
            game_equip_switch_list(data).await;
        }
        GameServer::MapProperty => {
            game_map_property(data).await;
        }
        GameServer::UnitIdle => {
            game_unit_idle(data).await;
        }
        GameServer::ScreenActiveEFST => {
            game_screen_active_esft(data).await;
        }
        GameServer::SkillTree => {
            game_skill_tree(data).await;
        }
        GameServer::ShortcutsKeyList => {
            game_shortcuts_key_list(data).await;
        }
        GameServer::LongParameterChange => {
            game_long_parameter_change(data).await;
        }
        GameServer::CharacterStatus => {
            game_character_status(data).await;
        }
        GameServer::UpdateStatus => {
            // SP_U<STAT> are used to update the amount of points necessary to increase that stat
            game_update_status(data).await;
        }
        GameServer::PartyInvitationState => {
            game_party_invitation_state(data).await;
        }
        GameServer::EquipWindowOpen => {
            game_equip_window_open(data).await;
        }
        GameServer::ConfigurationChange => {
            game_configuration_change(data).await;
        }
        GameServer::UnitChangedDir => {
            game_unit_changed_dir(data).await;
        }
        _ => {
            println!("Unknown packet id: {:x}", packet_id as u16);
        }
    }

    return true;
}

pub async fn game_listener(stream: &mut TcpStream) {
    let mut data_packet_id: u16 = u16::MAX;

    let mut total_read: usize = 0;
    let mut packet_len: usize = PACKET_HEADER_LEN as usize; // start on 2, to get packet id
    let mut buffer = [0; 16384];
    let header_size: usize = PACKET_HEADER_LEN as usize;
    let mut parse_len = false;
    let mut has_packet_len = false;

    println!("[game_listener] listening for packets..");

    let game_packets_len;
    unsafe {
        game_packets_len = GAME_PACKETS_LEN.as_ref().unwrap();
    }

    loop {
        stream.readable().await.expect("stream not readable");
        // print total read and packet len
        //println!("total read: {}, packet len: {}", total_read, packet_len);
        let read_result = stream.read(&mut buffer[total_read..packet_len]).await;
        match read_result {
            Ok(n) => {
                if n == 0 {
                    println!("Connection closed by server");
                    break;
                }

                total_read += n;

                //println!("total read: {}", total_read);

                // print the buffer as hexadecimal bytes, for debugging purposes
                // println!("{:02X?}", &buffer[0..total_read]);

                // read some data
                if !parse_len && data_packet_id != u16::MAX && total_read == packet_len {
                    let header_size = if has_packet_len {
                        PACKET_HEADER_LEN as usize + 2
                    } else {
                        PACKET_HEADER_LEN as usize
                    };

                    //println!("header size: {}, total read {}", header_size, total_read);
                    let mut input_message =
                        InputMessage::new(buffer[header_size..packet_len].to_vec());
                    let result =
                        game_packet_handler(stream, data_packet_id, &mut input_message).await;
                    match result {
                        true => {
                            println!("packet id {:x} handled", data_packet_id);
                            // reset packet_len to read the next packet
                            packet_len = PACKET_HEADER_LEN as usize;
                            data_packet_id = u16::MAX;
                            parse_len = false;
                            has_packet_len = false;
                            total_read = 0;
                        }
                        false => {
                            break;
                        }
                    }
                    continue;
                }

                if parse_len && total_read == packet_len {
                    packet_len =
                        u16::from_le_bytes([buffer[header_size], buffer[header_size + 1]]) as usize;
                    parse_len = false;

                    if packet_len == total_read {
                        // we read all the data, we should reset to next packet data!
                        packet_len = PACKET_HEADER_LEN as usize;
                        data_packet_id = u16::MAX;
                        parse_len = false;
                        has_packet_len = false;
                        total_read = 0;
                    }
                    continue;
                }

                if total_read == header_size {
                    // 2 bytes for packet id, 2 bytes for packet length
                    // parse packet id first
                    data_packet_id = u16::from_le_bytes([buffer[0], buffer[1]]);

                    println!("pckt id: {:x}", data_packet_id);

                    let result = game_packets_len.get(&data_packet_id);
                    match result {
                        Some(&len) => {
                            if len == u16::MAX {
                                // packet length will come on the next 2 bytes
                                packet_len = header_size + 2;
                                parse_len = true;
                                has_packet_len = true;
                            } else {
                                // packet length is known
                                packet_len += len as usize;
                            }
                        }
                        None => {
                            panic!("Unknown packet id: {:x}", data_packet_id);
                        }
                    }
                }
            }
            Err(e) => {
                println!("Failed to read from stream: {}", e);
                break;
            }
        }
    }
}

pub async fn initialize(
    ip: &str,
    port: u16,
    acc_id: u32,
    char_id: u32,
    login_id: u32,
    client_tick: u32,
    sex: u8,
) {
    unsafe {
        let mut game_packets_len = HashMap::new();
        game_packets_len.insert(GameServer::MapBlockList as u16, 4);
        game_packets_len.insert(GameServer::InventoryExpansionInfo as u16, 2);
        game_packets_len.insert(GameServer::NotifyChangeStatus as u16, 8);
        game_packets_len.insert(GameServer::AuthOk as u16, 11);
        game_packets_len.insert(GameServer::DisplayMessage as u16, u16::MAX);
        game_packets_len.insert(GameServer::ChangeMap as u16, 20);
        game_packets_len.insert(GameServer::ParameterChange as u16, 6);
        game_packets_len.insert(GameServer::CoupleStatus as u16, 12);
        game_packets_len.insert(GameServer::AtkRange as u16, 2);
        game_packets_len.insert(GameServer::MailUnread as u16, 1);
        game_packets_len.insert(GameServer::QuestsStateList as u16, u16::MAX);
        game_packets_len.insert(GameServer::SingleAchievementData as u16, 64);
        game_packets_len.insert(GameServer::AllAchievementsData as u16, u16::MAX);
        game_packets_len.insert(GameServer::WeightLimit as u16, 4);
        game_packets_len.insert(GameServer::SpriteChange as u16, 13);
        game_packets_len.insert(GameServer::InventoryStart as u16, u16::MAX);
        game_packets_len.insert(GameServer::InventoryType as u16, u16::MAX);
        game_packets_len.insert(GameServer::InventoryEnd as u16, 2);
        game_packets_len.insert(GameServer::EquipSwitchList as u16, u16::MAX);
        game_packets_len.insert(GameServer::MapProperty as u16, 6);
        game_packets_len.insert(GameServer::UnitIdle as u16, u16::MAX);
        game_packets_len.insert(GameServer::ScreenActiveEFST as u16, 26);
        game_packets_len.insert(GameServer::SkillTree as u16, u16::MAX);
        game_packets_len.insert(GameServer::ShortcutsKeyList as u16, 269);
        game_packets_len.insert(GameServer::LongParameterChange as u16, 10);
        game_packets_len.insert(GameServer::CharacterStatus as u16, 42);
        game_packets_len.insert(GameServer::UpdateStatus as u16, 3);
        game_packets_len.insert(GameServer::PartyInvitationState as u16, 1);
        game_packets_len.insert(GameServer::EquipWindowOpen as u16, 1);
        game_packets_len.insert(GameServer::ConfigurationChange as u16, 8);
        game_packets_len.insert(GameServer::UnitChangedDir as u16, 7);

        GAME_PACKETS_LEN = Some(game_packets_len);
    }

    let server_addr = format!("{}:{}", ip, port);
    let stream = TcpStream::connect(server_addr).await;
    match stream {
        Ok(mut stream) => {
            println!("Connected to game server: {}:{}", ip, port);
            // send connect to map server
            game_connect_map_server(&mut stream, acc_id, char_id, login_id, client_tick, sex).await;
            game_listener(&mut stream).await;
        }
        Err(e) => {
            println!("Failed to connect to server: {}", e);
        }
    }
}
