use std::fs;
use xxhash_rust::xxh64::xxh64;

#[derive(Debug)]
struct Chunk {
    contents : Vec<u8>,
    hash : u64
}

fn main() {
    let file_contents : Vec<u8> = match fs::read(r"D:\Rytansh\Trichic Games\StateArcheus\PatchDeltaPacker\Tests\TestData\ScanTest\GameConfig.json")
    {
        Ok(contents) => contents,
        Err(_) => return 
    };

    let mut chunked_file : Vec<Chunk> = Vec::new();
    
    let mut current_chunk : Chunk = Chunk {
        contents: Vec::with_capacity(32),
        hash: 1
    };

    for byte in &file_contents {
        if current_chunk.contents.len() == current_chunk.contents.capacity()
        {
            current_chunk.hash = xxh64(&current_chunk.contents, 1);
            chunked_file.push(current_chunk);
            current_chunk = Chunk {
                contents: Vec::with_capacity(32),
                hash: 1
            };
        }
        current_chunk.contents.push(*byte);
    };
    current_chunk.hash = xxh64(&current_chunk.contents, 1);
    chunked_file.push(current_chunk);

    println!("{chunked_file:?}")

}
