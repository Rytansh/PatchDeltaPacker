use std::fs;
use std::io;
use std::path::Path;

use crate::constants::PATCH_PACKAGES_PATH;
use crate::patcher::patch_package_gen::build_patch;
use crate::patcher::patch_structs::PatchPackage;

pub fn generate_patch(old_patch_root : &Path, new_patch_root : &Path) -> Result<PatchPackage, io::Error>
{
    let patch = build_patch(old_patch_root, new_patch_root)?;
    let output_path = Path::new(PATCH_PACKAGES_PATH);
    let output_file = output_path.join(format!("{}_to_{}.pdp", &patch.old_ver, &patch.new_ver));

    let bytes = serde_json::to_vec_pretty(&patch)?;
    fs::write(output_file, bytes)?;

    Ok(patch)
}