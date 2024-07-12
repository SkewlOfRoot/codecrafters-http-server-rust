use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::prelude::*;

pub fn gzip_string(input: &str) -> Vec<u8> {
    let mut compbody = Vec::new();

    GzEncoder::new(&mut compbody, Compression::default())
        .write_all(input.as_bytes())
        .unwrap();
    compbody
}
