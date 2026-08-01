use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::time::Instant;
use tokio::io;

use patch_packer::build::{concurrency::worker_pool::WorkerPool, manifests, patcher};
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, default_value_t = 1)]
    threads: usize,
}

#[derive(Subcommand)]
enum Commands {
    Manifest {
        #[arg(long)]
        root: PathBuf,
    },

    Build {
        #[arg(long)]
        old: PathBuf,

        #[arg(long)]
        new: PathBuf,

        #[arg(long)]
        output: PathBuf,
    },
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let worker_pool = WorkerPool::new(cli.threads);

    match cli.command {
        Commands::Manifest { root } => {
            let start = Instant::now();
            if let Err(err) = manifests::writer::generate_manifest(&root, &worker_pool).await {
                eprintln!("Failed to generate manifest: {err}");
                std::process::exit(1);
            }
            let elapsed = start.elapsed();
            println!("Manifest generation took {elapsed:.3?}.");
        }

        Commands::Build { old, new, output } => {
            let start = Instant::now();
            if let Err(err) =
                patcher::writer::generate_patch(&old, &new, &output, &worker_pool).await
            {
                eprintln!("Failed to generate patch: {err}");
                std::process::exit(1);
            }
            let elapsed = start.elapsed();
            println!("Patch generation took {elapsed:.3?}.");
        }
    }

    Ok(())
}
