use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::prelude::*;

pub fn gzip_string(input: &str) -> Vec<u8> {
    // Create a buffer to hold the compressed data
    // let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    // // Write the input string to the encoder
    // encoder
    //     .write_all(input.as_bytes())
    //     .expect("Failed to write data");
    // // Finish the compression process and retrieve the compressed data
    // let compressed_data = encoder.finish().expect("Failed to finish compression");
    // let strn = String::from(compressed_data).unwrap();
    // println!("{}", strn);
    // strn
    // Convert the compressed data to a hexadecimal string
    //hex::encode(compressed_data)

    let mut compbody = Vec::new();

    GzEncoder::new(&mut compbody, Compression::default())
        .write_all(input.as_bytes())
        .unwrap();
    compbody
}
