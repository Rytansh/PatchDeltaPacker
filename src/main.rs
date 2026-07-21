use std::path::Path;
use std::time::Instant;
mod build;
mod client;
mod constants;

use crate::build::concurrency::worker_pool::WorkerPool;
use crate::build::patcher::patch_package_gen;
use crate::client::installer::patch_installer;

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

    let patch_package = patch_package_gen::build_patch(
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

    let start2 = Instant::now();

    patch_installer::install_patch(
        patch_package,
        &worker_pool,
        Path::new(
            r"D:\Rytansh\Trichic Games\StateArcheus\PatchDeltaPacker\Testing\Installed Game\V1.1.0",
        ),
    )
    .await
    .unwrap();

    let elapsed2 = start2.elapsed();

    println!("Patch installation took {:.3?}", elapsed2);
}
