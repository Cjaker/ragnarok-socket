mod core;
mod network_message;
mod protocol_login;

use network_message::NetworkMessage;
use tokio::{io::AsyncWriteExt, net::TcpStream, runtime::Builder};

use std::io::{Error, ErrorKind};

pub static SERVER_ADDR: &str = "192.168.1.9:6900";
pub static MAX_USERNAME_LEN: u8 = 23 + 1; // 23 = len | 1 = null-terminator reserved
pub static MAX_PASSWORD_LEN: u8 = 32 + 1; // 32 = len | 1 = null-terminator reserved

async fn client_write_message(stream: &mut TcpStream, network_message: &NetworkMessage) -> Result<(), std::io::Error>{
    let write_result: Result<(), Error> = stream.write_all(&network_message.buffer[0..network_message.length as usize]).await;
    match write_result {
        Ok(_) => {
            Ok(())
        }
        Err(e) => {
            Err(Error::new(ErrorKind::Other, format!("Failed to send packet: {}", e)))
        }
    }
}

// 04 02 a2 cc 00 00 04 02 82 d1 2c 91 4f 5a d4 8f d9 6f cf 7e f4 cc 49 2d
async fn client_send_udpclhash(stream: &mut TcpStream) {
    // send udpclhash packet
    let client_md5: Vec<u8> = vec![
        0x82, 0xD1, 0x2C, 0x91, 0x4F, 0x5A, 0xD4, 0x8F, 0xD9, 0x6F, 0xCF, 0x7E, 0xF4, 0xCC, 0x49, 0x2D
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
    network_message.skip_bytes((MAX_USERNAME_LEN - 1 - username.len() as u8) as usize);

    // password, 24 bytes (length: MAX_USERNAME_LEN)
    network_message.add_string(password.as_str());
    // fill the missing bytes for password
    network_message.skip_bytes((MAX_USERNAME_LEN - 1 - password.len() as u8) as usize);

    // client_type
    network_message.add(0x00000002 as u8);

    let _ = client_write_message(stream, &network_message).await;
}

async fn initialize() {
    let stream = TcpStream::connect(SERVER_ADDR).await;
    match stream {
        Ok(mut stream) => {
            // send first packets
            client_send_udpclhash(&mut stream).await;
            client_send_reqauth(&mut stream, "test".to_string(), "test123".to_string()).await;
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
