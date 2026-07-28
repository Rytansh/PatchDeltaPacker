use patch_packer::build::concurrency::worker_pool::WorkerPool;
use patch_packer::build::config::config_reader;
use patch_packer::build::patcher::patch_ser;
use patch_packer::build::tooling::hasher;
use patch_packer::client::connection::connection_structs::Packet;
use patch_packer::client::connection::protocol::{receive_packet, send_packet};
use patch_packer::client::installer::patch_installer;
use sha2::Digest;
use std::io::{self, SeekFrom, Write};
use std::path::Path;
use tokio::fs;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> io::Result<()> {
    let worker_pool = WorkerPool::new(3);
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    let game_path = Path::new(
        r"D:\Rytansh\Trichic Games\StateArcheus\PatchDeltaPacker\Testing\Installed Game\V1.1.0",
    );
    let temp_path = game_path.join("patch.tmp");
    let resume_offset = fs::metadata(&temp_path)
        .await
        .map_or(0, |metadata| metadata.len());
    let current_game_version = config_reader::get_game_version(&game_path)?;

    send_packet(&mut stream, &Packet::Connection).await?;

    if matches!(receive_packet(&mut stream).await?, Packet::ConnectionAck) {
        println!("Connected to patch server.");
    } else {
        println!("Unexpected packet received. Closing connection.");
        send_packet(&mut stream, &Packet::ConnectionComplete).await?;
        return Ok(());
    }

    send_packet(
        &mut stream,
        &Packet::VersionRequest {
            current: current_game_version.clone(),
        },
    )
    .await?;

    if let Packet::VersionResponse { latest } = receive_packet(&mut stream).await? {
        if latest == current_game_version {
            println!("Game version is already up to date.");
            send_packet(&mut stream, &Packet::ConnectionComplete).await?;
            return Ok(());
        }
        println!("Requesting patch for version {latest}...");
        send_packet(
            &mut stream,
            &Packet::PatchRequest {
                from: current_game_version.clone(),
                to: latest.clone(),
                resume_offset,
            },
        )
        .await?;
    } else {
        println!("Unexpected packet received. Closing connection.");
        send_packet(&mut stream, &Packet::ConnectionComplete).await?;
        return Ok(());
    }

    if let Packet::PatchResponse {
        file,
        version,
        remaining_size,
        checksum,
    } = receive_packet(&mut stream).await?
    {
        println!(
            "An update is available ({:.1} MB).",
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
                    println!("Downloading {file}...");
                    send_packet(&mut stream, &Packet::PatchDownload).await?;
                    break;
                }

                "n" => {
                    println!("Cancelling update.");
                    send_packet(&mut stream, &Packet::ConnectionComplete).await?;
                    return Ok(());
                }
                _ => {
                    println!("An error occurred. Please try again.");
                }
            }
        }

        let mut temp_patch_file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&temp_path)
            .await?;

        temp_patch_file.seek(SeekFrom::Start(resume_offset)).await?;

        let mut remaining = remaining_size;
        let mut buffer = vec![0u8; 64 * 1024];

        while remaining > 0 {
            let bytes_read = stream.read(&mut buffer).await?;

            if bytes_read == 0 {
                println!("Error while downloading patch. Closing connection.");
                send_packet(&mut stream, &Packet::ConnectionComplete).await?;
                return Ok(());
            }

            temp_patch_file.write_all(&buffer[..bytes_read]).await?;
            remaining -= bytes_read as u64;
        }

        if matches!(receive_packet(&mut stream).await?, Packet::PatchComplete) {
            println!("Download complete. Verifying...");

            temp_patch_file.flush().await?;
            drop(temp_patch_file);

            let mut verification_file = tokio::fs::File::open(&temp_path).await?;
            let mut hasher = hasher::create_sha256();
            let mut buffer = vec![0u8; 64 * 1024];

            loop {
                let bytes_read = verification_file.read(&mut buffer).await?;

                if bytes_read == 0 {
                    break;
                }

                hasher.update(&buffer[..bytes_read]);
            }

            let temp_checksum: [u8; 32] = hasher.finalize().into();
            verification_file.flush().await?;
            drop(verification_file);
            if checksum != temp_checksum {
                println!("Updated patch files do not match - clearing progress.");
                fs::remove_file(&temp_path).await?;
                send_packet(&mut stream, &Packet::ConnectionComplete).await?;
                return Ok(());
            }
        } else {
            println!("An error occured while downloading the patch.");
            send_packet(&mut stream, &Packet::ConnectionComplete).await?;
            return Ok(());
        }
    }
    send_packet(&mut stream, &Packet::ConnectionComplete).await?;

    println!("Installing patch...");
    let patch = patch_ser::retrieve_patch(&temp_path).await?;
    patch_installer::install_patch(patch, &worker_pool, game_path).await?;
    fs::remove_file(temp_path).await?;
    println!("Patch installed!");
    Ok(())
}
