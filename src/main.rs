use std::{io::Write, net::TcpListener};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                _stream
                    .write_all(b"HTTP/1.1 200 OK\r\n\r\n")
                    .expect("Failed to write to stream");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
