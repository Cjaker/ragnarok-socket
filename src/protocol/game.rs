use std::collections::HashMap;

use num_enum::TryFromPrimitive;
use tokio::{io::AsyncReadExt, net::TcpStream};

use crate::{
    client::network::write_message, input_message::InputMessage, network_message::NetworkMessage,
    r#const::PACKET_HEADER_LEN,
};

#[derive(TryFromPrimitive)]
#[repr(u16)]
pub enum GameClient {
    ConnectMapServer = 0x0436,
    RequestAction = 0x0437
}

#[derive(TryFromPrimitive)]
#[repr(u16)]
pub enum GameServer {
    MapBlockList = 0x0283,
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

pub async fn game_request_action(
    stream: &mut TcpStream,
    target_id: u32,
    action_type: u8
) {
    let mut network_message = NetworkMessage::new();
    network_message.add(GameClient::RequestAction as u16);
    network_message.add(target_id);
    network_message.add(action_type);

    write_message(stream, &network_message).await;
}

pub async fn game_map_block_list(stream: &mut TcpStream, data: &mut InputMessage) {
    let acc_id = data.read_u32();
}

pub async fn game_packet_handler(stream: &mut TcpStream, packet_id: u16, data: &mut InputMessage) -> bool {
    println!("Packet ID: {:x}", packet_id);
    let packet_id =
        GameServer::try_from(packet_id).expect(format!("missing packet id {:x}", packet_id).as_str());

    match packet_id {
        GameServer::MapBlockList => {
            game_map_block_list(stream, data).await;
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

    println!("listening for packets..");

    let game_packets_len;
    unsafe {
        game_packets_len = GAME_PACKETS_LEN.as_ref().unwrap();
    }

    loop {
        stream.readable().await.expect("stream not readable");
        // print total read and packet len
        println!("total read: {}, packet len: {}", total_read, packet_len);
        let read_result = stream.read(&mut buffer[total_read..packet_len]).await;
        match read_result {
            Ok(n) => {
                if n == 0 {
                    println!("Connection closed by server");
                    break;
                }

                total_read += n;

                println!("total read: {}", total_read);

                // print the buffer as hexadecimal bytes, for debugging purposes
                // println!("{:02X?}", &buffer[0..total_read]);

                // read some data
                if !parse_len && data_packet_id != u16::MAX && total_read == packet_len {
                    let header_size = if has_packet_len {
                        PACKET_HEADER_LEN as usize + 2
                    } else {
                        PACKET_HEADER_LEN as usize
                    };

                    println!("header size: {}, total read {}", header_size, total_read);
                    let mut input_message =
                        InputMessage::new(buffer[header_size..packet_len].to_vec());
                    let result = game_packet_handler(stream, data_packet_id, &mut input_message).await;
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
