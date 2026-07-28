use patch_packer::build::patcher::patch_history;
use patch_packer::client::connection::connection_structs::{
    ErrorCode, Packet, PendingDownload, Session,
};
use patch_packer::client::connection::protocol::{receive_packet, send_packet};
use patch_packer::constants::{PATCH_HISTORY_RELATIVE_PATH, PATCH_PACKAGES_PATH};
use std::io;
use std::io::SeekFrom;
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let patch_history_path = Path::new(PATCH_PACKAGES_PATH).join(PATCH_HISTORY_RELATIVE_PATH);
    loop {
        let (mut stream, client_addr) = listener.accept().await?;
        let mut session = Session { download: None };

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

                Packet::PatchRequest {
                    from,
                    to,
                    resume_offset,
                } => {
                    let patch_entry =
                        patch_history::get_patch_entry(&patch_history_path, &from, &to)?;
                    if resume_offset > patch_entry.size {
                        send_packet(
                            &mut stream,
                            &Packet::Error {
                                code: ErrorCode::FatalError,
                            },
                        )
                        .await?;
                        continue;
                    }
                    session.download = Some(PendingDownload {
                        patch_entry: patch_entry.clone(),
                        resume_offset,
                    });
                    send_packet(
                        &mut stream,
                        &Packet::PatchResponse {
                            file: patch_entry.file,
                            version: to,
                            remaining_size: patch_entry.size - resume_offset,
                            checksum: patch_entry.checksum,
                        },
                    )
                    .await?;
                }

                Packet::PatchDownload => {
                    let Some(download) = &session.download else {
                        send_packet(
                            &mut stream,
                            &Packet::Error {
                                code: ErrorCode::DownloadInfoNotFound,
                            },
                        )
                        .await?;
                        continue;
                    };

                    let mut file = File::open(
                        Path::new(PATCH_PACKAGES_PATH).join(Path::new(&download.patch_entry.file)),
                    )
                    .await?;

                    file.seek(SeekFrom::Start(download.resume_offset)).await?;

                    let mut buffer = vec![0u8; 64 * 1024];

                    loop {
                        let bytes = file.read(&mut buffer).await?;

                        if bytes == 0 {
                            println!("Bytes exhausted.");
                            break;
                        }

                        stream.write_all(&buffer[..bytes]).await?;
                    }

                    send_packet(&mut stream, &Packet::PatchComplete).await?;
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
