use clap::Parser;
use std::io::{self, SeekFrom};
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use patch_packer::build::patcher::{self, structs::PatchEntry};
use patch_packer::client::connection::{
    protocol::{receive_packet, send_packet},
    structs::Packet,
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
    loop {
        let packet = receive_packet(&mut stream).await?;

        match packet {
            Packet::Connection => {
                handle_connection(&mut stream).await?;
            }

            Packet::VersionRequest { current } => {
                handle_version_request(&mut stream, patch_directory, current).await?;
            }

            Packet::PatchRequest { from, to } => {
                handle_patch_request(&mut stream, patch_directory, from, to).await?;
            }

            Packet::PatchDownload {
                patch,
                resume_offset,
            } => {
                handle_patch_download(&mut stream, patch, resume_offset, &patch_directory).await?;
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
    patch_directory: &Path,
    from: String,
    to: String,
) -> io::Result<()> {
    let patches = patcher::history::get_patch_chain(patch_directory, &from, &to)?;

    send_packet(
        stream,
        &Packet::PatchResponse {
            patches,
            target: to,
        },
    )
    .await
}

async fn handle_patch_download(
    stream: &mut TcpStream,
    patch: PatchEntry,
    resume_offset: u64,
    patch_directory: &Path,
) -> io::Result<()> {
    if resume_offset > patch.size {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The resume progress exceeds the patch size.",
        ));
    }
    let mut file = File::open(patch_directory.join(patch.file)).await?;

    file.seek(SeekFrom::Start(resume_offset)).await?;

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
