#[derive(num_enum::TryFromPrimitive)]
#[repr(u16)]
pub enum LoginServer {
    AuthOk = 0x0AC4,
    AuthResult = 0x0081,
}

#[repr(u16)]
pub enum LoginClient {
    UDPCLHASH = 0x0204,
    REQAUTH = 0x0064,
}

use crate::{
    client::network::write_message, r#const::{LOGIN_SERVER_ADDR, PACKET_HEADER_LEN}, enums, input_message::InputMessage, network_message::NetworkMessage, protocol::game
};
use std::collections::HashMap;
use tokio::{io::AsyncReadExt, net::TcpStream};

pub static mut LOGIN_PACKETS_LEN: Option<HashMap<u16, u16>> = None;
pub static MAX_CREDENTIAL_LEN: u8 = 23; // 23 = len | 1 = null-terminator reserved

async fn login_packet_handler(packet_id: u16, data: &mut InputMessage) -> bool {
    println!("Packet ID: {:x}", packet_id);
    let packet_id = LoginServer::try_from(packet_id)
        .expect(format!("missing packet id {:x}", packet_id).as_str());

    match packet_id {
        LoginServer::AuthOk => {
            login_auth_ok(data).await;
            return false; // break listener loop
        }
        LoginServer::AuthResult => {
            login_auth_result(data).await;
        }
    }

    return true;
}

// parse packets
async fn login_auth_ok(data: &mut InputMessage) {
    let login_id: u32 = data.read_u32();
    let acc_id: u32 = data.read_u32();
    let login_id_2: u32 = data.read_u32();
    let ip: u32 = data.read_u32();
    data.skip_bytes(24); // null bytes
    let unk = data.read_u16();
    let gender = data.read_u8();
    let web_token = data.read_string(None);

    loop {
        let server_ip = data.read_u32();
        let server_port = data.read_u16();
        let server_name = data.read_string(Some(20));
        let server_users = data.read_u16();
        let server_type = data.read_u16();
        let server_is_new = data.read_u16();
        data.skip_bytes(128); // unknown bytes

        // print ip, port and name, in a readable format
        // ip to string
        let ip_str = format!(
            "{}.{}.{}.{}",
            (server_ip & 0xFF),
            (server_ip >> 8 & 0xFF),
            (server_ip >> 16 & 0xFF),
            (server_ip >> 24 & 0xFF)
        );

        println!("Server: {}:{} - {}", ip_str, server_port, server_name);

        if data.is_eof() {
            tokio::spawn(async move {
                game::initialize(&ip_str, server_port, login_id, login_id_2, acc_id, gender).await;
            });
            break;
        }
    }

    println!("Done reading servers");
}

async fn login_auth_result(data: &mut InputMessage) {
    let result = enums::AuthResult::try_from(data.read_u8());
    match result {
        Ok(result) => match result {
            enums::AuthResult::ServerClosed => {
                println!("Server closed");
            }
            enums::AuthResult::AlreadyLoggedWithId => {
                println!("Already logged with id");
            }
            enums::AuthResult::AlreadyOnline => {
                println!("Already online");
            }
        },
        Err(e) => {
            println!("Failed to parse auth result: {}", e);
        }
    }
}

// 04 02 a2 cc 00 00 04 02 82 d1 2c 91 4f 5a d4 8f d9 6f cf 7e f4 cc 49 2d
async fn client_send_udpclhash(stream: &mut TcpStream) {
    // send udpclhash packet
    let client_md5: Vec<u8> = vec![
        0x82, 0xD1, 0x2C, 0x91, 0x4F, 0x5A, 0xD4, 0x8F, 0xD9, 0x6F, 0xCF, 0x7E, 0xF4, 0xCC, 0x49,
        0x2D,
    ];

    let mut network_message = NetworkMessage::new();
    network_message.add(LoginClient::UDPCLHASH as u16);
    for x in client_md5 {
        network_message.add(x);
    }

    let _ = write_message(stream, &network_message).await;
}

// 55 bytes total?
async fn client_send_reqauth(stream: &mut TcpStream, username: String, password: String) {
    let mut network_message = NetworkMessage::new();
    network_message.add(LoginClient::REQAUTH as u16);
    // 4 unknown bytes?
    network_message.add(0x80000001 as u32);

    // username, 24 bytes (length: MAX_USERNAME_LEN)
    network_message.add_string(username.as_str());
    // fill the missing bytes for username
    network_message.skip_bytes((MAX_CREDENTIAL_LEN - username.len() as u8) as usize);

    // password, 24 bytes (length: MAX_USERNAME_LEN)
    network_message.add_string(password.as_str());
    // fill the missing bytes for password
    network_message.skip_bytes((MAX_CREDENTIAL_LEN - password.len() as u8) as usize);

    // client_type
    network_message.add(0x00000002 as u8);

    let _ = write_message(stream, &network_message).await;
}

async fn login_listener(stream: &mut TcpStream) {
    let mut data_packet_id: u16 = 0xFFFF;

    let mut total_read: usize = 0;
    let mut packet_len: usize = PACKET_HEADER_LEN as usize; // start on 2, to get packet id
    let mut buffer = [0; 16384];
    let header_size: usize = PACKET_HEADER_LEN as usize;
    let mut parse_len = false;
    let mut has_packet_len = false;

    println!("listening for packets..");

    let login_packets_len;
    unsafe {
        login_packets_len = LOGIN_PACKETS_LEN.as_ref().unwrap();
    }

    loop {
        stream.readable().await.expect("stream not readable");
        let read_result = stream.read(&mut buffer[total_read..packet_len]).await;
        match read_result {
            Ok(n) => {
                if n == 0 {
                    println!("Connection closed by server");
                    break;
                }

                total_read += n;

                // print the buffer as hexadecimal bytes, for debugging purposes
                // println!("{:02X?}", &buffer[0..total_read]);

                // read some data
                if !parse_len && data_packet_id != 0xFFFF && total_read == packet_len {
                    let header_size = if has_packet_len {
                        PACKET_HEADER_LEN as usize + 2
                    } else {
                        PACKET_HEADER_LEN as usize
                    };

                    let mut input_message =
                        InputMessage::new(buffer[header_size..packet_len].to_vec());
                    let result = login_packet_handler(data_packet_id, &mut input_message).await;
                    match result {
                        true => {
                            // reset packet_len to read the next packet
                            packet_len = PACKET_HEADER_LEN as usize;
                            data_packet_id = 0;
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
                    continue;
                }

                if total_read == header_size {
                    // 2 bytes for packet id, 2 bytes for packet length
                    // parse packet id first
                    data_packet_id = u16::from_le_bytes([buffer[0], buffer[1]]);

                    //println!("pckt id: {}", data_packet_id);

                    let result = login_packets_len.get(&data_packet_id);
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

pub async fn initialize() {
    // initialize login packets size
    unsafe {
        let mut login_packets_len = HashMap::new();
        login_packets_len.insert(LoginServer::AuthOk as u16, u16::MAX as u16);
        login_packets_len.insert(LoginServer::AuthResult as u16, 1);

        LOGIN_PACKETS_LEN = Some(login_packets_len);
    }

    let stream = TcpStream::connect(LOGIN_SERVER_ADDR).await;
    match stream {
        Ok(mut stream) => {
            // send first packets
            client_send_udpclhash(&mut stream).await;
            client_send_reqauth(&mut stream, "test".to_string(), "test123".to_string()).await;
            tokio::spawn(async move {
                login_listener(&mut stream).await;
            })
            .await
            .expect("Failed to spawn listener");
        }
        Err(e) => {
            println!("Failed to connect to server: {}", e);
        }
    }
}
