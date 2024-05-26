use std::collections::HashMap;
use tokio::{io::AsyncReadExt, net::TcpStream};

use crate::{
    client::network::write_message,
    input_message::InputMessage,
    network_message::NetworkMessage,
    r#const::PACKET_HEADER_LEN,
};

use super::{game, login};

#[repr(u16)]
pub enum CharListClient {
    ReqToConnect = 0x0065,
    ReqCharList = 0x9A1,
    CharSelect = 0x0066,
}

#[derive(num_enum::TryFromPrimitive)]
#[repr(u16)]
pub enum CharListServer {
    WindowData = 0x082D,
    CharsData = 0x006B,
    Notify = 0x09A0,
    BanCharacter = 0x020D,
    PinCodeState = 0x08B9,
    AckCharInfoPerPage = 0x0B72,
    MapData = 0xAC5,
    MapServerNotReady = 0x0840
}

pub static mut CHAR_LIST_PACKETS_LEN: Option<HashMap<u16, u16>> = None;
static mut CHAR_LIST_ACC_ID: u32 = 0;
static mut CHAR_LIST_LOGIN_ID: u32 = 0;
static mut CHAR_LIST_SEX: u8 = 0;

pub async fn char_list_window_data(data: &mut InputMessage) {
    let min_chars = data.read_u8();
    let premium_chars = data.read_u8();
    let billing_chars = data.read_u8();
    let producible_chars = data.read_u8();
    let max_chars = data.read_u8();
    data.skip_bytes(20); // unused bytes

    println!("max chars: {}", max_chars);
}

pub async fn parse_char_info(data: &mut InputMessage) {
    loop {
        let gid = data.read_u32();
        let exp = data.read_u64();
        let money = data.read_u32();
        let job_exp = data.read_u64();
        let job_level = data.read_u32();
        let body_state = data.read_u32();
        let health_state = data.read_u32();
        let effect_state = data.read_u32();
        let virtue = data.read_u32();
        let honor = data.read_u32();
        let jobpoint = data.read_u16();
        let hp = data.read_u64();
        let maxhp = data.read_u64();
        let sp = data.read_u64();
        let maxsp = data.read_u64();
        let speed = data.read_u16();
        let job = data.read_u16();
        let head = data.read_u16();

        let mut body = data.read_u16();
        let weapon = data.read_u16();
        let level = data.read_u16();
        let sppoint = data.read_u16();
        let accessory = data.read_u16();
        let shield = data.read_u16();
        let accessory2 = data.read_u16();
        let accessory3 = data.read_u16();
        let headpalette = data.read_u16();
        let bodypalette = data.read_u16();

        let char_name = data.read_string(Some(24));
        println!("char name: {}", char_name);
        let stat_str = data.read_u8();
        let stat_agi = data.read_u8();
        let stat_vit = data.read_u8();
        let stat_int = data.read_u8();
        let stat_dex = data.read_u8();
        let stat_luk = data.read_u8();
        let char_num = data.read_u8();
        let hair_color = data.read_u8();
        let is_changed_char_name = data.read_u16();

        let map_name = data.read_string(Some(16));
        println!("map name: {}", map_name);

        let del_rev_date = data.read_u32();

        let robe_palette = data.read_u32();
        let chr_slot_change_cnt = data.read_u32();
        let chr_name_change_cnt = data.read_u32();
        let sex = data.read_u8();

        if data.is_eof() {
            println!("End of chars data");
            break;
        }
    }
}

pub async fn char_list_chars_data(data: &mut InputMessage) {
    let max_chars = data.read_u8();
    let min_chars = data.read_u8();
    let premium_chars = data.read_u8();
    data.skip_bytes(20);

    parse_char_info(data).await;
}

pub async fn char_list_notify(data: &mut InputMessage) {
    let nb_pages_count = data.read_u32();

    println!("nb pages count: {}", nb_pages_count);
}

pub async fn char_list_ban_character(data: &mut InputMessage) {
    loop {
        if data.is_eof() {
            println!("End of ban character data");
            break;
        }

        let character_id = data.read_u32();
        if character_id != 0 {
            let ban_str = data.read_string(Some(20));
        }
    }
}

pub async fn char_list_pin_code_state(data: &mut InputMessage) {
    let pin_code_seed = data.read_u32();
    let pin_code_account_id = data.read_u32();
    let pin_code_state = data.read_u16();

    // print all vars
    println!(
        "pin code seed: {}, account id: {}, state: {}",
        pin_code_seed, pin_code_account_id, pin_code_state
    );
}

pub async fn read_bytes(stream: &mut TcpStream, len: usize) -> Vec<u8> {
    let mut buffer = vec![0; len];
    let mut total_read = 0;

    while total_read < len {
        let read_result = stream.read(&mut buffer[total_read..len]).await;
        match read_result {
            Ok(n) => {
                total_read += n;
            }
            Err(e) => {
                println!("Failed to read from stream: {}", e);
                break;
            }
        }
    }

    return buffer;
}

pub async fn char_list_packet_handler(
    stream: &mut TcpStream,
    packet_id: u16,
    data: &mut InputMessage,
) -> bool {
    println!("Packet ID: {:x}, data len: {}", packet_id, data.length);
    let packet_id = CharListServer::try_from(packet_id)
        .expect(format!("missing packet id {:x}", packet_id).as_str());

    match packet_id {
        CharListServer::WindowData => {
            char_list_window_data(data).await;
            return true;
        }
        CharListServer::CharsData => {
            char_list_chars_data(data).await;
            return true;
        }
        CharListServer::Notify => {
            char_list_notify(data).await;
            return true;
        }
        CharListServer::BanCharacter => {
            char_list_ban_character(data).await;
            return true;
        }
        CharListServer::PinCodeState => {
            char_list_pin_code_state(data).await;
            // send req char list
            char_list_reqcharlist(stream).await;
            return true;
        }
        CharListServer::AckCharInfoPerPage => {
            char_list_ack_char_info_per_page(data).await;
            // select random character for test purposes
            char_list_char_select(stream, 0).await;
            return true;
        }
        CharListServer::MapServerNotReady => {
            char_list_map_server_not_ready(data).await;
            return true;
        }
        CharListServer::MapData => {
            char_list_map_data(data).await;
            return false;
        }
    }
}

pub async fn game_listener(stream: &mut TcpStream) {
    let mut data_packet_id: u16 = u16::MAX;

    let mut total_read: usize = 0;
    let mut packet_len: usize = PACKET_HEADER_LEN as usize; // start on 2, to get packet id
    let mut buffer = [0; 16384];
    let header_size: usize = PACKET_HEADER_LEN as usize;
    let mut parse_len = false;
    let mut has_packet_len = false;

    println!("listening for packets..");

    let game_packets_len;
    unsafe {
        game_packets_len = CHAR_LIST_PACKETS_LEN.as_ref().unwrap();
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
                        char_list_packet_handler(stream, data_packet_id, &mut input_message).await;
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
                            println!("ending character list listener");
                            break;
                        }
                    }
                    continue;
                }

                if parse_len && total_read == packet_len {
                    packet_len = u16::from_le_bytes([buffer[header_size], buffer[header_size + 1]]) as usize;
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

                    //println!("pckt id: {:x}", data_packet_id);

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

// client->server packets
pub async fn char_list_reqconnect(
    stream: &mut TcpStream,
    login_id: u32,
    login_id_2: u32,
    acc_id: u32,
    sex: u8,
) {
    println!("Sending char list request");

    let mut network_message = NetworkMessage::new();
    network_message.add(CharListClient::ReqToConnect as u16);
    network_message.add(acc_id);
    network_message.add(login_id);
    network_message.add(login_id_2);
    network_message.add(0 as u16); // unknown
    network_message.add(sex);

    write_message(stream, &network_message).await;

    // after sending this packet, the server will send 4 bytes as account id
    let account_id_vec = read_bytes(stream, 4).await;
    let account_id = u32::from_le_bytes([
        account_id_vec[0],
        account_id_vec[1],
        account_id_vec[2],
        account_id_vec[3],
    ]);

    println!("Received account id from server: {}", account_id);
}

pub async fn char_list_reqcharlist(stream: &mut TcpStream) {
    println!("Sending char list request");

    let mut network_message = NetworkMessage::new();
    network_message.add(CharListClient::ReqCharList as u16);
    write_message(stream, &network_message).await;
}

pub async fn char_list_char_select(stream: &mut TcpStream, index: u8) {
    println!("Sending char list select");

    let mut network_message = NetworkMessage::new();
    network_message.add(CharListClient::CharSelect as u16);
    network_message.add(index);
    write_message(stream, &network_message).await;
}

pub async fn char_list_ack_char_info_per_page(data: &mut InputMessage) {
    parse_char_info(data).await;
}

pub async fn char_list_map_server_not_ready(data: &mut InputMessage) {
    let unk = data.read_u16();
    data.skip_bytes(20);
}

pub async fn char_list_map_data(data: &mut InputMessage) {
    let char_id = data.read_u32();
    let map_name = data.read_string(Some(16));
    let map_ip = data.read_u32();
    let map_port = data.read_u16();
    data.skip_bytes(128); // unknown bytes

    println!("char id: {}, map name: {}, map ip: {}, map port: {}", char_id, map_name, map_ip, map_port);

    // reverse ip and convert ip and port to server addr
    let map_ip_str = format!(
        "{}.{}.{}.{}",
        (map_ip & 0xFF),
        (map_ip >> 8 & 0xFF),
        (map_ip >> 16 & 0xFF),
        (map_ip >> 24 & 0xFF)
    );

    let acc_id;
    let login_id;
    let sex;
    unsafe {
        acc_id = CHAR_LIST_ACC_ID;
        login_id = CHAR_LIST_LOGIN_ID;
        sex = CHAR_LIST_SEX;
    }

    // print all vars
    println!(
        "char id: {}, map name: {}, map ip: {}, map port: {}",
        char_id, map_name, map_ip_str, map_port
    );

    tokio::spawn(async move {
        game::initialize(&map_ip_str, map_port, acc_id, char_id, login_id, 111111111, sex).await;
    });
}

pub async fn initialize(ip: &str, port: u16, login_id: u32, login_id_2: u32, acc_id: u32, sex: u8) {
    // print login id's
    println!("login id: {:x}, login id 2: {:x}", login_id, login_id_2);
    unsafe {
        CHAR_LIST_ACC_ID = acc_id;
        CHAR_LIST_LOGIN_ID = login_id;
        CHAR_LIST_SEX = sex;

        let mut char_list_packets_len = HashMap::new();
        char_list_packets_len.insert(CharListServer::WindowData as u16, u16::MAX);
        char_list_packets_len.insert(CharListServer::CharsData as u16, u16::MAX);
        char_list_packets_len.insert(CharListServer::Notify as u16, 4);
        char_list_packets_len.insert(CharListServer::BanCharacter as u16, u16::MAX);
        char_list_packets_len.insert(CharListServer::PinCodeState as u16, 10);
        char_list_packets_len.insert(CharListServer::AckCharInfoPerPage as u16, u16::MAX);
        char_list_packets_len.insert(CharListServer::MapData as u16, 154);
        char_list_packets_len.insert(CharListServer::MapServerNotReady as u16, 22);

        CHAR_LIST_PACKETS_LEN = Some(char_list_packets_len);
    }

    let server_addr = format!("{}:{}", ip, port);
    let stream = TcpStream::connect(server_addr).await;
    match stream {
        Ok(mut stream) => {
            println!("Connected to character list server: {}:{}", ip, port);
            // send char list request
            char_list_reqconnect(&mut stream, login_id, login_id_2, acc_id, sex).await;
            game_listener(&mut stream).await;
        }
        Err(e) => {
            println!("Failed to connect to server: {}", e);
        }
    }
}
