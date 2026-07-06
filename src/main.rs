use std::path::{Path};

mod manifests;
mod config;
mod tooling;
mod chunker;
mod constants;
use crate::manifests::{manifest_gen, manifest_reader};


fn main() {
    manifest_gen::write_manifest(Path::new(r"D:\Rytansh\Trichic Games\StateArcheus\PatchDeltaPacker\Testing\Dummy Version Data\V1.1.0")).unwrap();
    let manifest = manifest_reader::get_manifest(Path::new(r"D:\Rytansh\Trichic Games\StateArcheus\PatchDeltaPacker\Testing\Dummy Version Data\V1.2.0")).unwrap();

    println!("{manifest:?}");
    
}
