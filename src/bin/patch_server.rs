use clap::Parser;
use std::path::{Path, PathBuf};
use std::{io, io::SeekFrom};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use patch_packer::build::patcher;
use patch_packer::client::connection::{
    protocol::{receive_packet, send_packet},
    structs::{ErrorCode, Packet, PendingDownload, Session},
};

#[derive(Parser)]
#[command(name = "patch_server", about = "Hosts patch files for clients.")]
struct Cli {
    #[arg(long)]
    packages: PathBuf,

    #[arg(long, default_value = "127.0.0.1:8080")]
    port: String,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let listener = TcpListener::bind(&cli.port).await?;

    loop {
        let (stream, _) = listener.accept().await?;

        run_session(stream, &cli.packages).await?;
    }
}

async fn run_session(mut stream: TcpStream, patch_directory: &Path) -> io::Result<()> {
    let mut session = Session { download: None };

    loop {
        let packet = receive_packet(&mut stream).await?;

        match packet {
            Packet::Connection => {
                handle_connection(&mut stream).await?;
            }

            Packet::VersionRequest { current } => {
                handle_version_request(&mut stream, patch_directory, current).await?;
            }

            Packet::PatchRequest {
                from,
                to,
                resume_offset,
            } => {
                handle_patch_request(
                    &mut stream,
                    &mut session,
                    patch_directory,
                    from,
                    to,
                    resume_offset,
                )
                .await?;
            }

            Packet::PatchDownload => {
                handle_patch_download(&mut stream, &session, patch_directory).await?;
            }

            Packet::ConnectionComplete => {
                println!("Client disconnected.");
                break;
            }

            _ => {}
        }
    }

    Ok(())
}
async fn handle_connection(stream: &mut TcpStream) -> io::Result<()> {
    println!("Connection established.");

    send_packet(stream, &Packet::ConnectionAck).await
}

async fn handle_version_request(
    stream: &mut TcpStream,
    patch_directory: &Path,
    current: String,
) -> io::Result<()> {
    println!("Client currently has version {current}");

    let latest = patcher::history::get_latest_version(patch_directory)?;

    send_packet(stream, &Packet::VersionResponse { latest }).await
}

async fn handle_patch_request(
    stream: &mut TcpStream,
    session: &mut Session,
    patch_directory: &Path,
    from: String,
    to: String,
    resume_offset: u64,
) -> io::Result<()> {
    let patch_entry = patcher::history::get_patch_entry(patch_directory, &from, &to)?;

    if resume_offset > patch_entry.size {
        send_packet(
            stream,
            &Packet::Error {
                code: ErrorCode::FatalError,
            },
        )
        .await?;

        return Ok(());
    }

    session.download = Some(PendingDownload {
        patch_entry: patch_entry.clone(),
        resume_offset,
    });

    send_packet(
        stream,
        &Packet::PatchResponse {
            file: patch_entry.file,
            version: to,
            remaining_size: patch_entry.size - resume_offset,
            checksum: patch_entry.checksum,
        },
    )
    .await
}

async fn handle_patch_download(
    stream: &mut TcpStream,
    session: &Session,
    patch_directory: &Path,
) -> io::Result<()> {
    let Some(download) = &session.download else {
        send_packet(
            stream,
            &Packet::Error {
                code: ErrorCode::DownloadInfoNotFound,
            },
        )
        .await?;

        return Ok(());
    };

    let mut file = File::open(patch_directory.join(&download.patch_entry.file)).await?;

    file.seek(SeekFrom::Start(download.resume_offset)).await?;

    let mut buffer = vec![0; 64 * 1024];

    loop {
        let bytes = file.read(&mut buffer).await?;

        if bytes == 0 {
            break;
        }

        stream.write_all(&buffer[..bytes]).await?;
    }

    send_packet(stream, &Packet::PatchComplete).await
}
