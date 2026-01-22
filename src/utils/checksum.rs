use base64::{engine, Engine};
use sha1::{Digest, Sha1};
use std::{
    fs::File,
    io::{self, Read},
};

pub fn check_checksum_of_file(path: String, checksum: String) -> anyhow::Result<bool> {
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let digest = Sha1::digest(buf);
    let base64 = engine::general_purpose::STANDARD.encode(digest);

    if base64 == checksum {
        Ok(true)
    } else {
        Ok(false)
    }
}
