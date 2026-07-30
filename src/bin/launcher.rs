use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use sha2::Digest;
use std::io::{self, SeekFrom, Write};
use std::path::{Path, PathBuf};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::{fs, fs::OpenOptions};

use patch_packer::build::concurrency::worker_pool::WorkerPool;
use patch_packer::build::{config, patcher, tooling};
use patch_packer::client::connection::{
    protocol::{receive_packet, send_packet},
    structs::Packet,
};
use patch_packer::client::installation;
use patch_packer::constants::{CHUNK_SIZE, TEMPORARY_PATCH_PATH};

#[derive(Parser)]
#[command(name = "launcher", about = "Downloads and installs game updates.")]
struct Cli {
    #[arg(long, default_value_t = 1)]
    threads: usize,

    #[arg(long)]
    game: PathBuf,
}

struct DownloadInfo {
    file_name: String,
    remaining_size: u64,
    checksum: [u8; 32],
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let worker_pool = WorkerPool::new(cli.threads);

    run_launcher(cli.game, &worker_pool).await
}

async fn run_launcher(game_path: PathBuf, worker_pool: &WorkerPool) -> io::Result<()> {
    //compile metadata here like resume offset etc
    let temp_patch_path = game_path.join(TEMPORARY_PATCH_PATH);
    if let Some(parent) = temp_patch_path.parent() {
        fs::create_dir_all(parent).await?;
    }
    let resume_offset = fs::metadata(&temp_patch_path)
        .await
        .map_or(0, |metadata| metadata.len());

    let mut stream = connect_to_server().await?;

    let result: io::Result<()> = async {
        if !find_update(&mut stream, &game_path, resume_offset).await? {
            return Err(io::Error::from(io::ErrorKind::NotFound));
        }

        let Some(download_info) = get_patch_from_server(&mut stream).await? else {
            return Err(io::Error::from(io::ErrorKind::NotFound));
        };

        download_patch(&mut stream, &temp_patch_path, resume_offset, &download_info).await?;

        if !verify_patch(&temp_patch_path, &download_info).await? {
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        }

        Ok(())
    }
    .await;

    close_connection(&mut stream).await;

    match result {
        Ok(()) => {}
        Err(err) => {
            return Ok(());
        }
    }

    install_patch(&temp_patch_path, &game_path, worker_pool).await
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

async fn find_update(
    stream: &mut TcpStream,
    game_path: &Path,
    resume_offset: u64,
) -> io::Result<bool> {
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
                resume_offset,
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

async fn get_patch_from_server(stream: &mut TcpStream) -> Result<Option<DownloadInfo>, io::Error> {
    if let Packet::PatchResponse {
        file,
        version,
        remaining_size,
        checksum,
    } = receive_packet(stream).await?
    {
        println!(
            "Version {version}: An update is available ({:.1} MB).",
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
                    return Ok(Some(DownloadInfo {
                        file_name: file,
                        remaining_size,
                        checksum,
                    }));
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
    info: &DownloadInfo,
) -> io::Result<()> {
    send_packet(stream, &Packet::PatchDownload).await?;
    let mut temp_patch_file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(&temp_patch_path)
        .await?;

    temp_patch_file.seek(SeekFrom::Start(resume_offset)).await?;

    let mut remaining = info.remaining_size;
    let mut buffer = vec![0; CHUNK_SIZE];

    let total_size = resume_offset + info.remaining_size;

    println!("Downloading {}...", info.file_name);

    let progress = ProgressBar::new(total_size);

    progress.set_position(resume_offset);

    progress.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] \
         {bytes}/{total_bytes} ({bytes_per_sec}, ETA {eta})",
        )
        .unwrap()
        .progress_chars("#>-"),
    );

    while remaining > 0 {
        let bytes_read = stream.read(&mut buffer).await?;
        if bytes_read == 0 {
            println!("EOF");
            break;
        }
        temp_patch_file.write_all(&buffer[..bytes_read]).await?;
        remaining -= bytes_read as u64;
        progress.inc(bytes_read as u64);
    }

    if !matches!(receive_packet(stream).await?, Packet::PatchComplete) {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Am error occurred when downloading the patch: PATCH_COMPLETE_NOT_RECEIVED",
        ));
    }
    progress.finish_with_message("Download complete!");
    temp_patch_file.flush().await?;
    Ok(())
}

async fn verify_patch(temp_patch_path: &Path, info: &DownloadInfo) -> io::Result<bool> {
    let mut verification_file = fs::File::open(temp_patch_path).await?;
    let mut hasher = tooling::hasher::create_sha256();
    let mut buffer = vec![0; CHUNK_SIZE];

    println!("Verifying patch...");

    loop {
        let bytes_read = verification_file.read(&mut buffer).await?;

        if bytes_read == 0 {
            break;
        }

        hasher.update(&buffer[..bytes_read]);
    }

    let temp_checksum: [u8; 32] = hasher.finalize().into();
    if info.checksum != temp_checksum {
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
) -> io::Result<()> {
    println!("Installing patch...");
    let patch = patcher::writer::retrieve_patch(temp_patch_path).await?;
    installation::installer::install_patch(patch, &worker_pool, game_path).await?;
    fs::remove_file(temp_patch_path).await?;
    println!("Patch installed!");
    Ok(())
}
