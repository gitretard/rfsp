use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    os::unix::prelude::OsStrExt,
    path::Path,
    process,
    fs::{File},
    io::{Read}
};

pub fn craft_metadata(filepath: &str) -> [u8; 512]{
    let fp = Path::new(filepath);
    let fm = match fp.metadata() {
        Ok(r) => r,
        Err(a) => {
            println!("Failed to get file metadata: {}", a);
            process::exit(1)
        }
    };
    let mut meta: [u8; 512] = [0; 512];
    // Hope it works
    meta[0..8].copy_from_slice(&fm.len().to_be_bytes());
    // Get random number
    let r: u32 = 0; // u32 should be more than enough
    meta[8..12].copy_from_slice(&r.to_be_bytes());
    if let Some(filen) = fp.file_name() {
        let len = filen.as_bytes().len();
        meta[12..12 + len].copy_from_slice(filen.as_bytes())
    }
    meta // for whatever reason this doesnt work. try it,fp
}

pub fn hash<T>(t: T) -> u64 // Wtf is this
where
    T: Hash,
{
    let mut hasher = DefaultHasher::new();
    t.hash(&mut hasher);
    hasher.finish()
}

pub fn craft_data_packet(idempt:u32,mut f:&File) -> Option<[u8;512]>{ // Will be executed after metadata handshake
    let mut data: [u8;512] = [0;512];
    // Start the loop
    let mut fpart: [u8;498] = [0;498]; // I probably forgot to change somethign somewhere
    // I need to use the same fs::File instance to uhhhhhh nvm
    data[9..13].copy_from_slice(&idempt.to_be_bytes());
    let bytes_read = f.read(&mut fpart);
    if bytes_read.unwrap() == 0 {
        return None;
    }
    data[14..512].copy_from_slice(&fpart);
    let hash = hash(&data[8..512]);
    data[0..8].copy_from_slice(&hash.to_be_bytes());
    Some(data)
}

pub fn craft_done_packet() -> [u8;512]{
    let mut data: [u8;512] = [0;512];
    data[9] = 2;
    data
}