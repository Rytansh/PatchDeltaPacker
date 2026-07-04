mod manifest_gen;
mod chunk_builder;
mod chunk_extractor;

fn main() {
    let manifest = manifest_gen::build_manifest_file(r"D:\Rytansh\Trichic Games\StateArcheus\PatchDeltaPacker\Tests\TestData\ScanTest\GameConfig.json").unwrap();

    println!("{manifest:?}");
}
