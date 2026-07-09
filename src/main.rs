use std::path::{Path};

mod manifests;
mod config;
mod tooling;
mod chunker;
mod constants;
mod patcher;
use crate::manifests::{manifest_gen, manifest_reader};
use crate::patcher::{patch_plan_gen};


fn main() {
    
    let writemanifestv1 = manifest_gen::write_manifest(Path::new(r"D:\Rytansh\Trichic Games\StateArcheus\PatchDeltaPacker\Testing\Dummy Version Data\V1.1.0")).unwrap();
    let writemanifestv2 = manifest_gen::write_manifest(Path::new(r"D:\Rytansh\Trichic Games\StateArcheus\PatchDeltaPacker\Testing\Dummy Version Data\V1.1.1")).unwrap();
    let manifestv1 = manifest_reader::get_manifest(Path::new(r"D:\Rytansh\Trichic Games\StateArcheus\PatchDeltaPacker\Testing\Dummy Version Data\V1.1.0")).unwrap();
    let manifestv2 = manifest_reader::get_manifest(Path::new(r"D:\Rytansh\Trichic Games\StateArcheus\PatchDeltaPacker\Testing\Dummy Version Data\V1.1.1")).unwrap();
    
    let plan = patch_plan_gen::create_patch_plan(&manifestv1, &manifestv2).unwrap();

    println!("{plan:?}");
    
}
