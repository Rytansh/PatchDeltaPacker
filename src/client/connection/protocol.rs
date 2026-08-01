use bincode::config;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::client::connection::structs::Packet;

pub async fn receive_packet(stream: &mut TcpStream) -> io::Result<Packet> {
    let length = stream.read_u32().await?;
    let mut buffer = vec![0; length as usize];
    stream.read_exact(&mut buffer).await?;

    let (packet, _) = bincode::serde::decode_from_slice(&buffer, config::standard())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    Ok(packet)
}

pub async fn send_packet(stream: &mut TcpStream, packet: &Packet) -> io::Result<()> {
    let bytes = bincode::serde::encode_to_vec(packet, config::standard())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    stream.write_u32(bytes.len() as u32).await?;
    stream.write_all(&bytes).await?;

    Ok(())
}
