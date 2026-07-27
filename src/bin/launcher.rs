use patch_packer::build::config::config_reader;
use patch_packer::client::connection::connection_structs::Packet;
use patch_packer::client::connection::protocol::{receive_packet, send_packet};
use std::io::{self, Write};
use std::path::Path;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    let game_path = Path::new(
        r"D:\Rytansh\Trichic Games\StateArcheus\PatchDeltaPacker\Testing\Installed Game\V1.1.0",
    );
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
            },
        )
        .await?;
    } else {
        println!("Unexpected packet received. Closing connection.");
        send_packet(&mut stream, &Packet::ConnectionComplete).await?;
        return Ok(());
    }

    if let Packet::PatchResponse {
        name,
        size,
        checksum,
    } = receive_packet(&mut stream).await?
    {
        println!(
            "An update is available ({:.1} MB).",
            size as f64 / (1024.0 * 1024.0)
        );

        let mut input = String::new();
        loop {
            input.clear();
            print!("Download now? (y/n): ");
            io::stdout().flush()?;
            io::stdin().read_line(&mut input)?;

            match input.trim().to_lowercase().as_str() {
                "y" => {
                    println!("Downloading {name}...");
                    // Tell the server you're ready.
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
    }

    send_packet(&mut stream, &Packet::ConnectionComplete).await?;
    Ok(())
}
