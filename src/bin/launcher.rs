use clap::Parser;
use sha2::Digest;
use std::io::{self, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tokio::fs::{self, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::net::TcpStream;

use patch_packer::build::{
    concurrency::worker_pool::WorkerPool,
    patcher::{self, structs::PatchEntry},
    {config, tooling},
};
use patch_packer::client::{
    connection::{
        protocol::{receive_packet, send_packet},
        structs::Packet,
    },
    installation::{self, progress::UpdateProgress},
};

use patch_packer::constants::{CHUNK_SIZE, TEMPORARY_PATCH_PATH};

#[derive(Parser)]
#[command(name = "launcher", about = "Downloads and installs game updates.")]
struct Cli {
    #[arg(long, default_value_t = 1)]
    threads: usize,

    #[arg(long)]
    game: PathBuf,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let worker_pool = WorkerPool::new(cli.threads);

    run_launcher(cli.game, &worker_pool).await
}

async fn run_launcher(game_path: PathBuf, worker_pool: &WorkerPool) -> io::Result<()> {
    let temp_patch_path = game_path.join(TEMPORARY_PATCH_PATH);
    if let Some(parent) = temp_patch_path.parent() {
        fs::create_dir_all(parent).await?;
    }

    let initial_resume_offset = fs::metadata(&temp_patch_path)
        .await
        .map_or(0, |metadata| metadata.len());

    let mut stream = connect_to_server().await?;

    let result: io::Result<()> = async {
        if !find_update(&mut stream, &game_path).await? {
            return Ok(());
        }

        let Some(patches) =
            get_patches_from_server(&mut stream, &temp_patch_path, initial_resume_offset).await?
        else {
            return Ok(());
        };

        let start = Instant::now();

        let progress = Arc::new(UpdateProgress::new(patches.len()));

        for (index, patch) in patches.iter().enumerate() {
            progress.begin_patch(index);
            let patch_resume_offset = fs::metadata(&temp_patch_path)
                .await
                .map_or(0, |metadata| metadata.len());

            download_patch(
                &mut stream,
                &temp_patch_path,
                patch_resume_offset,
                patch,
                progress.as_ref(),
            )
            .await?;

            if !verify_patch(&temp_patch_path, patch, progress.as_ref()).await? {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Downloaded patch failed verification.",
                ));
            }
            progress.finish_verification();

            install_patch(
                &temp_patch_path,
                &game_path,
                worker_pool,
                Arc::clone(&progress),
            )
            .await?;
        }

        progress.finish();

        println!("Patch installed!");
        println!("Installed in {:.3?}.", start.elapsed());
        Ok(())
    }
    .await;

    close_connection(&mut stream).await;

    result
}

async fn connect_to_server() -> io::Result<TcpStream> {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    send_packet(&mut stream, &Packet::Connection).await?;

    if matches!(receive_packet(&mut stream).await?, Packet::ConnectionAck) {
        println!("Successfully connected to patch server.");
    } else {
        return Err(io::Error::new(
            io::ErrorKind::ConnectionAborted,
            "Connection was not established properly.",
        ));
    }

    Ok(stream)
}

async fn close_connection(stream: &mut TcpStream) {
    if let Err(err) = send_packet(stream, &Packet::ConnectionComplete).await {
        eprintln!("Failed to close connection: {err}");
    }
}

async fn find_update(stream: &mut TcpStream, game_path: &Path) -> io::Result<bool> {
    let current_game_version = config::reader::get_game_version(game_path)?;

    send_packet(
        stream,
        &Packet::VersionRequest {
            current: current_game_version.clone(),
        },
    )
    .await?;

    if let Packet::VersionResponse { latest } = receive_packet(stream).await? {
        if latest == current_game_version {
            println!("Game version is already up to date.");
            return Ok(false);
        }
        send_packet(
            stream,
            &Packet::PatchRequest {
                from: current_game_version.clone(),
                to: latest,
            },
        )
        .await?;
    } else {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "An unexpected error occurred: VERSION_RESPONSE_NOT_RECEIVED",
        ));
    }
    Ok(true)
}

async fn get_patches_from_server(
    stream: &mut TcpStream,
    temp_patch_path: &Path,
    resume_offset: u64,
) -> Result<Option<Vec<PatchEntry>>, io::Error> {
    if let Packet::PatchResponse { patches, target } = receive_packet(stream).await? {
        let total_size: u64 = patches.iter().map(|patch| patch.size).sum();
        let mut resume_offset = resume_offset;
        if resume_offset > total_size {
            println!("The patch may have been corrupted. Deleting patch...");
            fs::remove_file(&temp_patch_path).await?;
            resume_offset = 0;
        }

        let remaining_size = total_size - resume_offset;
        if remaining_size == 0 {
            println!("Current download already complete.");
            return Ok(Some(patches));
        }
        println!(
            "Version {target}: An update is available ({:.1} MB).",
            remaining_size as f64 / (1024.0 * 1024.0)
        );

        let mut input = String::new();
        loop {
            input.clear();
            print!("Download now? (y/n): ");
            io::stdout().flush()?;
            io::stdin().read_line(&mut input)?;

            match input.trim().to_lowercase().as_str() {
                "y" => {
                    return Ok(Some(patches));
                }
                "n" => {
                    println!("Cancelling update.");
                    return Ok(None);
                }
                _ => {
                    println!("The input was invalid. Please try again.");
                }
            }
        }
    } else {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "An unexpected error occurred: PATCH_RESPONSE_NOT_RECEIVED",
        ))
    }
}

async fn download_patch(
    stream: &mut TcpStream,
    temp_patch_path: &Path,
    resume_offset: u64,
    patch: &PatchEntry,
    progress: &UpdateProgress,
) -> io::Result<()> {
    if resume_offset > patch.size {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Resume offset exceeds patch size.",
        ));
    }

    progress.set_message("Downloading...");

    send_packet(
        stream,
        &Packet::PatchDownload {
            patch: patch.clone(),
            resume_offset,
        },
    )
    .await?;

    let mut temp_patch_file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(&temp_patch_path)
        .await?;

    temp_patch_file.seek(SeekFrom::Start(resume_offset)).await?;

    let mut remaining = patch.size - resume_offset;
    let mut buffer = vec![0; CHUNK_SIZE];

    while remaining > 0 {
        let chunk = remaining.min(buffer.len() as u64) as usize;

        let bytes_read = stream.read(&mut buffer[..chunk]).await?;

        if bytes_read == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Connection closed before patch download completed.",
            ));
        }

        temp_patch_file.write_all(&buffer[..bytes_read]).await?;

        remaining -= bytes_read as u64;

        let downloaded = patch.size - remaining;
        progress.download_progress(downloaded, patch.size);
    }

    if !matches!(receive_packet(stream).await?, Packet::PatchComplete) {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Am error occurred when downloading the patch increment: UNEXPECTED_PACKET_RESULT",
        ));
    }
    temp_patch_file.flush().await?;
    Ok(())
}

async fn verify_patch(
    temp_patch_path: &Path,
    patch: &PatchEntry,
    progress: &UpdateProgress,
) -> io::Result<bool> {
    let mut verification_file = fs::File::open(temp_patch_path).await?;
    let mut hasher = tooling::hasher::create_sha256();
    let mut buffer = vec![0; CHUNK_SIZE];

    progress.set_message("Verifying...");

    loop {
        let bytes_read = verification_file.read(&mut buffer).await?;

        if bytes_read == 0 {
            break;
        }

        hasher.update(&buffer[..bytes_read]);
    }

    let temp_checksum: [u8; 32] = hasher.finalize().into();
    if patch.checksum != temp_checksum {
        println!("Updated patch files do not match - clearing progress.");
        fs::remove_file(&temp_patch_path).await?;
        return Ok(false);
    }
    Ok(true)
}

async fn install_patch(
    temp_patch_path: &Path,
    game_path: &Path,
    worker_pool: &WorkerPool,
    progress: Arc<UpdateProgress>,
) -> io::Result<()> {
    progress.set_message("Retrieving...");
    let patch = patcher::writer::retrieve_patch(temp_patch_path).await?;
    progress.set_message("Installing...");
    installation::installer::install_patch(patch, &worker_pool, game_path, progress).await?;
    fs::remove_file(temp_patch_path).await?;
    Ok(())
}
