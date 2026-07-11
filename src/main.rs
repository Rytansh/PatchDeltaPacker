use std::path::{Path};

mod manifests;
mod config;
mod tooling;
mod chunker;
mod constants;
mod patcher;
use crate::patcher::{patch_ser};


fn main() {
    
    let package = patch_ser::generate_patch(Path::new(r"D:\Rytansh\Trichic Games\StateArcheus\PatchDeltaPacker\Testing\Dummy Version Data\V1.1.0"), Path::new(r"D:\Rytansh\Trichic Games\StateArcheus\PatchDeltaPacker\Testing\Dummy Version Data\V1.1.1")).unwrap();

    println!("{package:?}");
}
