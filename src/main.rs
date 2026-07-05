mod manifest_gen;
mod chunk_builder;
mod chunk_extractor;
mod directory_scanner;
mod config_reader;
use std::path::{Path};

fn main() {
    let manifest = manifest_gen::build_manifest(Path::new(r"D:\Rytansh\Trichic Games\StateArcheus\PatchDeltaPacker\Testing\Dummy Version Data\V1.1.0")).unwrap();
    let manifest2 = manifest_gen::build_manifest(Path::new(r"D:\Rytansh\Trichic Games\StateArcheus\PatchDeltaPacker\Testing\Dummy Version Data\V1.2.0")).unwrap();

    println!("{manifest:?}");
    println!("{manifest2:?}");
}
