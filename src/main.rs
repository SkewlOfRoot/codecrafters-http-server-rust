use http_request::{HttpRequest, HttpRequestType};
use http_response::{HttpResponse, HttpResponseBuilder, HttpStatusCode};
use http_server_starter_rust::ThreadPool;
use itertools::Itertools;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::{env, fs};

mod compressor;
mod http_header;
mod http_request;
mod http_response;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    let pool = ThreadPool::new(8);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| handle_connection(stream));
    }
}

fn handle_connection(mut stream: TcpStream) {
    let request = read_request(&mut stream).unwrap();
    println!("REQUEST:\r\n{:#?}", request);

    let response = generate_response(&request);
    write_response(stream, response);
}

fn read_request(stream: &mut TcpStream) -> Result<HttpRequest, String> {
    let mut read_buff = [0; 1024];
    match stream.read(&mut read_buff) {
        Ok(bytes_read) => {
            if let Ok(request) = String::from_utf8(read_buff[..bytes_read].to_vec()) {
                Ok(HttpRequest::from_str(&request))
            } else {
                Err(String::from("Received non-UTF8 data."))
            }
        }
        Err(e) => Err(format!("Failed to read bytes from stream: {}", e)),
    }
}

fn generate_response(request: &HttpRequest) -> HttpResponse {
    let response: HttpResponse;
    if request.request_path == "/" {
        response = gen_root_response();
    } else if request.request_path.starts_with("/echo") {
        response = gen_echo_response(request);
    } else if request.request_path == "/user-agent" {
        response = gen_user_agent_response(request);
    } else if request.request_path.starts_with("/files") {
        response = gen_files_response(request);
    } else {
        response = HttpResponse::not_found()
    }
    println!("RESPONSE:\r\n{:#?}", response);
    response
}

fn gen_root_response() -> HttpResponse {
    HttpResponseBuilder::new(HttpStatusCode::Ok)
        .build()
        .unwrap()
}

fn gen_echo_response(request: &HttpRequest) -> HttpResponse {
    let value = request.request_path.split('/').last().unwrap();

    let collect_vec = &request
        .headers
        .iter()
        .filter(|h| h.name.to_lowercase() == "accept-encoding")
        .collect_vec();
    let accept_encoding_header = collect_vec.first();

    if accept_encoding_header.is_some_and(|h| h.value.to_lowercase().contains("gzip")) {
        let compressed_value = compressor::gzip_string(value);

        HttpResponseBuilder::new(HttpStatusCode::Ok)
            .content_encoding("gzip")
            .body(compressed_value)
            .build()
            .unwrap()
    } else {
        HttpResponseBuilder::new(HttpStatusCode::Ok)
            .body(value.as_bytes().to_vec())
            .build()
            .unwrap()
    }
}

fn gen_user_agent_response(request: &HttpRequest) -> HttpResponse {
    let collect_vec = &request
        .headers
        .iter()
        .filter(|x| x.name.to_lowercase() == "user-agent")
        .collect_vec();
    let user_agent = collect_vec.first();
    if let Some(u) = user_agent {
        HttpResponseBuilder::new(HttpStatusCode::Ok)
            .body(u.value.as_bytes().to_vec())
            .build()
            .unwrap()
    } else {
        HttpResponse::not_found()
    }
}

fn gen_files_response(request: &HttpRequest) -> HttpResponse {
    let env_args: Vec<String> = env::args().collect();
    let file_dir = env_args[2].clone();
    let file_name = request.request_path.split('/').last().unwrap();
    let file_path = Path::new(file_dir.as_str()).join(file_name);
    match request.request_type {
        HttpRequestType::Get => {
            let file_content = fs::read(file_path);

            if let Ok(content) = file_content {
                HttpResponseBuilder::new(HttpStatusCode::Ok)
                    .content_type("application/octet-stream")
                    .body(content)
                    .build()
                    .unwrap()
            } else {
                HttpResponse::not_found()
            }
        }
        HttpRequestType::Post => {
            fs::write(file_path, request.body.as_ref().unwrap()).unwrap();
            HttpResponseBuilder::new(HttpStatusCode::Created)
                .build()
                .unwrap()
        }
        _ => HttpResponse::not_found(),
    }
}

fn write_response(mut stream: TcpStream, response: HttpResponse) {
    stream
        .write_all(&response.output())
        .expect("Failed to write to stream");
}
