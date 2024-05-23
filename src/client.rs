pub mod network {
    use std::io::{Error, ErrorKind};
    use tokio::{io::AsyncWriteExt, net::TcpStream};

    use crate::network_message::NetworkMessage;

    pub async fn write_message(
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
}
