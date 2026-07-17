use crate::concurrency::worker_pool::WorkerPool;
use std::path::Path;
use std::time::Instant;

mod chunker;
mod concurrency;
mod config;
mod constants;
mod manifests;
mod patcher;
mod tooling;

use crate::patcher::patch_ser;

#[tokio::main]
async fn main() {
    let worker_pool = WorkerPool::new(3);

    // let start = Instant::now();
    //
    // let manifest = manifest_ser::write_manifest(
    //     Path::new(
    //         r"D:\Rytansh\Trichic Games\StateArcheus\PatchDeltaPacker\Testing\Version Data\V1.1.0",
    //     ),
    //     &worker_pool,
    // )
    // .await
    // .unwrap();
    //
    // let elapsed = start.elapsed();
    //
    // println!("Manifest took {:.3?}", elapsed);

    let start = Instant::now();

    let patch_package = patch_ser::generate_patch(
        Path::new(
            r"D:\Rytansh\Trichic Games\StateArcheus\PatchDeltaPacker\Testing\Version Data\V1.1.3",
        ),
        Path::new(
            r"D:\Rytansh\Trichic Games\StateArcheus\PatchDeltaPacker\Testing\Version Data\V1.1.4",
        ),
        &worker_pool,
    )
    .await
    .unwrap();

    let elapsed = start.elapsed();

    println!("Patch package took {:.3?}", elapsed);
}
