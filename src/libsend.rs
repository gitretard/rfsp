use std::{os::unix::prelude::OsStrExt, 
    path::Path, 
    process,
};

pub fn craft_metadata(filepath: &str) -> [u8; 512] {
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
    if let Some(filen) = fp.file_name() {
        let len = filen.as_bytes().len();
        meta[8..8 + len].copy_from_slice(filen.as_bytes())
    }
    return meta;
}
