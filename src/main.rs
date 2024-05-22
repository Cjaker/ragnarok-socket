mod core;
mod network_message;
mod protocol_login;
mod input_message;
mod enums;

use network_message::NetworkMessage;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    runtime::Builder,
};

use std::io::{Error, ErrorKind};

use crate::input_message::InputMessage;

pub static SERVER_ADDR: &str = "192.168.1.9:6900";
pub static MAX_CREDENTIAL_LEN: u8 = 23; // 23 = len | 1 = null-terminator reserved

async fn client_write_message(
    stream: &mut TcpStream,
    network_message: &NetworkMessage,
) -> Result<(), std::io::Error> {
    let write_result: Result<(), Error> = stream
        .write_all(&network_message.buffer[0..network_message.length as usize])
        .await;
    match write_result {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::new(
            ErrorKind::Other,
            format!("Failed to send packet: {}", e),
        )),
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
    network_message.add(protocol_login::client::UDPCLHASH as u16);
    for x in client_md5 {
        network_message.add(x);
    }

    let _ = client_write_message(stream, &network_message).await;
}

// 55 bytes total?
async fn client_send_reqauth(stream: &mut TcpStream, username: String, password: String) {
    let mut network_message = NetworkMessage::new();
    network_message.add(protocol_login::client::REQAUTH as u16);
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

    let _ = client_write_message(stream, &network_message).await;
}

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

        println!("Server Name: {}", server_name);
        break;
    }
}

async fn login_auth_result(data: &mut InputMessage) {
    println!("okay!");
    let result = enums::AuthResult::try_from(data.read_u8());
    match result {
        Ok(result) => {
            match result {
                enums::AuthResult::ServerClosed => {
                    println!("Server closed");
                }
                enums::AuthResult::AlreadyLoggedWithId => {
                    println!("Already logged with id");
                }
                enums::AuthResult::AlreadyOnline => {
                    println!("Already online");
                }
            }
        }
        Err(e) => {
            println!("Failed to parse auth result: {}", e);
        }
    }
}

async fn login_packet_handler(packet_id: u16, data: &mut InputMessage) {
    println!("Packet ID: {:x}", packet_id);
    let packet_id = protocol_login::LoginServer::try_from(packet_id)
        .expect(format!("missing packet id {:x}", packet_id).as_str());
    match packet_id {
        protocol_login::LoginServer::AuthOk => {
            println!("Received AUTH packet!");
            login_auth_ok(data).await;
        }
        protocol_login::LoginServer::AuthResult => {
            login_auth_result(data).await;
        }
    }
}

async fn listener(stream: &mut TcpStream) {
    let mut data_packet_id: u16 = 0xFFFF;
    let mut data_packet_length: u16 = 0;

    let mut total_read: usize = 0;
    let mut packet_len: usize = 4; // start on 4, to get packet header
    let mut buffer = [0; 16384];
    let header_size: usize = 4;

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
                println!("{:02X?}", &buffer[0..total_read]);

                // read some data
                if data_packet_id != 0xFFFF && total_read == packet_len {
                    println!(
                        "packet id {}, total read {}, packet len {}",
                        data_packet_id, total_read, packet_len
                    );

                    let mut input_message = InputMessage::new(buffer[header_size..packet_len].to_vec());
                    login_packet_handler(data_packet_id, &mut input_message).await;
                    total_read = 0;
                    packet_len = 4;
                    data_packet_id = 0;
                    data_packet_length = 0;
                    continue;
                }

                println!("total read: {}", total_read);
                if total_read >= 2 && total_read <= 3 {
                    let mut input_message = InputMessage::new(buffer[0..total_read].to_vec());
                    data_packet_id = input_message.read_u16();
                    login_packet_handler(data_packet_id, &mut input_message).await;
                    total_read = 0;
                    packet_len = 4;
                    data_packet_id = 0;
                    data_packet_length = 0;
                    continue;
                }

                if total_read == header_size {
                    // 2 bytes for packet id, 2 bytes for packet length
                    // parse packet id first
                    data_packet_id = u16::from_le_bytes([buffer[0], buffer[1]]);
                    // parse packet length
                    data_packet_length = u16::from_le_bytes([buffer[2], buffer[3]]);

                    packet_len = data_packet_length as usize;
                    println!("pckt id: {}, pckt len: {}, packet_len: {}", data_packet_id, data_packet_length, packet_len);
                }
            }
            Err(e) => {
                println!("Failed to read from stream: {}", e);
                break;
            }
        }
    }
}

async fn initialize() {
    let stream = TcpStream::connect(SERVER_ADDR).await;
    match stream {
        Ok(mut stream) => {
            // send first packets
            client_send_udpclhash(&mut stream).await;
            client_send_reqauth(&mut stream, "test".to_string(), "test123".to_string()).await;
            tokio::spawn(async move {
                listener(&mut stream).await;
            })
            .await
            .expect("Failed to spawn listener");
        }
        Err(e) => {
            println!("Failed to connect to server: {}", e);
        }
    }
}

fn main() {
    let rt = Builder::new_multi_thread()
        .worker_threads(core::WORKER_THREADS as usize)
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        initialize().await;
    })
}
