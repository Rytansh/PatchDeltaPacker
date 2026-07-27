use patch_packer::build::patcher::patch_history;
use patch_packer::client::connection::connection_structs::Packet;
use patch_packer::client::connection::protocol::{receive_packet, send_packet};
use patch_packer::constants::{PATCH_HISTORY_RELATIVE_PATH, PATCH_PACKAGES_PATH};
use std::io;
use std::path::Path;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let patch_history_path = Path::new(PATCH_PACKAGES_PATH).join(PATCH_HISTORY_RELATIVE_PATH);
    loop {
        let (mut stream, client_addr) = listener.accept().await?;

        loop {
            let packet = receive_packet(&mut stream).await?;

            match packet {
                Packet::Connection => {
                    println!("Connection established.");

                    send_packet(&mut stream, &Packet::ConnectionAck).await?;
                }

                Packet::VersionRequest { current } => {
                    println!("Client currently has version {}", current);
                    let latest_version = patch_history::get_latest_version(&patch_history_path)?;
                    send_packet(
                        &mut stream,
                        &Packet::VersionResponse {
                            latest: latest_version,
                        },
                    )
                    .await?;
                }

                Packet::PatchRequest { from, to } => {
                    let patch_entry =
                        patch_history::get_patch_entry(&patch_history_path, &from, &to)?;
                    send_packet(
                        &mut stream,
                        &Packet::PatchResponse {
                            name: patch_entry.file,
                            size: patch_entry.size,
                            checksum: patch_entry.checksum,
                        },
                    )
                    .await?;
                }

                Packet::ConnectionComplete => {
                    println!("Client disconnected.");
                    break;
                }

                _ => {}
            }
        }
    }
}
