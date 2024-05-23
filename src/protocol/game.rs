
use tokio::net::TcpStream;

use crate::{client::network::{self, write_message}, network_message::NetworkMessage};

#[repr(u16)]
pub enum CharListServer {
    REQTOCONNECT = 0x0065,
}

pub async fn listener() {
    loop {
        println!("Listening for game server packets");
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

pub async fn char_list_reqconnect(stream: &mut TcpStream, login_id: u32, login_id_2: u32, acc_id: u32, sex: u8) {
    println!("Sending char list request");

    let mut network_message = NetworkMessage::new();
    network_message.add(CharListServer::REQTOCONNECT as u16);
    network_message.add(acc_id);
    network_message.add(login_id);
    network_message.add(login_id_2);
    network_message.add(0 as u16); // unknown
    network_message.add(sex);

    write_message(stream, &network_message).await;
}

pub async fn initialize(ip: &str, port: u16, login_id: u32, login_id_2: u32, acc_id: u32, sex: u8) {
    let server_addr = format!("{}:{}", ip, port);
    let stream = TcpStream::connect(server_addr).await;
    match stream {
        Ok(mut stream) => {
            println!("Connected to game server: {}:{}", ip, port);
            // send char list request
            char_list_reqconnect(&mut stream, login_id, login_id_2, acc_id, sex).await;
            listener().await;
        }
        Err(e) => {
            println!("Failed to connect to server: {}", e);
        }
    }
}
