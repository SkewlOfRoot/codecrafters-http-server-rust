use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::{env, fs};

use itertools::Itertools;

use http_server_starter_rust::ThreadPool;

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
    response
}

fn gen_root_response() -> HttpResponse {
    HttpResponse::ok(None)
}

fn gen_echo_response(request: &HttpRequest) -> HttpResponse {
    let value = request.request_path.split('/').last().unwrap();
    HttpResponse::ok(Some(value.to_string()))
}

fn gen_user_agent_response(request: &HttpRequest) -> HttpResponse {
    let collect_vec = &request
        .headers
        .iter()
        .filter(|x| x.name.to_lowercase() == "user-agent")
        .collect_vec();
    let user_agent = collect_vec.first();
    if let Some(u) = user_agent {
        HttpResponse::ok(Some(String::from(&u.value)))
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
            let file_content = fs::read_to_string(file_path);

            if let Ok(content) = file_content {
                HttpResponseBuilder::new()
                    .status_code(HttpStatusCode::Ok)
                    .content_type("application/octet-stream")
                    .body(&content)
                    .build()
                    .unwrap()
            } else {
                HttpResponse::not_found()
            }
        }
        HttpRequestType::Post => {
            fs::write(file_path, request.body.as_ref().unwrap()).unwrap();
            HttpResponseBuilder::new()
                .status_code(HttpStatusCode::Created)
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

#[derive(Debug)]
struct HttpRequest {
    request_type: HttpRequestType,
    request_path: String,
    headers: Vec<HttpHeader>,
    body: Option<String>,
}

#[derive(Debug)]
enum HttpRequestType {
    Get,
    Put,
    Post,
    Delete,
    Patch,
}

impl HttpRequest {
    fn from_str(request_str: &str) -> HttpRequest {
        let lines = &mut request_str.split('\n');
        let first_line = lines.next().unwrap();
        let mut values = first_line.split(' ');
        let mut headers: Vec<HttpHeader> = Vec::new();

        for header_line in lines {
            if header_line == "\r" {
                break;
            }
            let header_vals = header_line.splitn(2, ':').map(|x| x.trim()).collect_vec();
            headers.push(HttpHeader::new(
                header_vals.first().unwrap(),
                header_vals.last().unwrap(),
            ))
        }

        let (_, b) = request_str.split_once("\r\n\r\n").unwrap();

        HttpRequest {
            request_type: HttpRequestType::from_str(values.next().unwrap()).unwrap(),
            request_path: values.next().unwrap().to_string(),
            headers,
            body: Some(b.trim().to_string()),
        }
    }
}

impl HttpRequestType {
    fn from_str(type_str: &str) -> Result<HttpRequestType, String> {
        let lowercase = type_str.to_lowercase();
        match lowercase.as_str() {
            "get" => Ok(HttpRequestType::Get),
            "put" => Ok(HttpRequestType::Put),
            "post" => Ok(HttpRequestType::Post),
            "delete" => Ok(HttpRequestType::Delete),
            "patch" => Ok(HttpRequestType::Patch),
            _ => Err(format!(
                "Could not parse value '{}' to HttpRequestType.",
                lowercase
            )),
        }
    }
}

struct HttpResponse {
    version: String,
    status_code: HttpStatus,
    headers: Vec<HttpHeader>,
    body: String,
}

struct HttpStatus {
    code: u16,
    description: &'static str,
}

enum HttpStatusCode {
    Ok,
    Created,
    NotFound,
}

impl HttpStatusCode {
    fn status(&self) -> HttpStatus {
        match self {
            HttpStatusCode::Ok => HttpStatus {
                code: 200,
                description: "OK",
            },
            HttpStatusCode::Created => HttpStatus {
                code: 201,
                description: "Created",
            },
            HttpStatusCode::NotFound => HttpStatus {
                code: 404,
                description: "Not Found",
            },
        }
    }
}

#[derive(Debug)]
struct HttpHeader {
    name: String,
    value: String,
}

impl HttpHeader {
    fn new(name: &str, value: &str) -> HttpHeader {
        HttpHeader {
            name: String::from(name),
            value: String::from(value),
        }
    }

    fn output(self) -> String {
        format!("{}: {}", self.name, self.value)
    }
}

impl HttpResponse {
    fn ok(body: Option<String>) -> HttpResponse {
        HttpResponseBuilder::new()
            .status_code(HttpStatusCode::Ok)
            .content_type("text/plain")
            .body(body.unwrap_or_default().as_str())
            .build()
            .unwrap()
    }

    fn not_found() -> HttpResponse {
        HttpResponseBuilder::new()
            .status_code(HttpStatusCode::NotFound)
            .build()
            .unwrap()
    }

    fn output(self) -> Vec<u8> {
        let mut response_lines: Vec<String> = vec![format!(
            "{} {} {}",
            self.version, self.status_code.code, self.status_code.description
        )];

        for header in self.headers {
            response_lines.push(header.output())
        }

        let mut response_str = response_lines.join("\r\n");
        response_str.push_str("\r\n\r\n");

        if !self.body.is_empty() {
            response_str.push_str(self.body.as_str());
        }

        println!("RESPONSE:\r\n{}", response_str);
        response_str.as_bytes().to_vec()
    }
}

struct HttpResponseBuilder {
    http_version: Option<String>,
    status_code: Option<HttpStatusCode>,
    content_type: Option<String>,
    body: Option<String>,
}

impl HttpResponseBuilder {
    fn new() -> Self {
        HttpResponseBuilder {
            http_version: None,
            status_code: None,
            content_type: None,
            body: None,
        }
    }

    fn status_code(mut self, status_code: HttpStatusCode) -> Self {
        self.status_code = Some(status_code);
        self
    }

    fn content_type(mut self, content_type: &str) -> Self {
        self.content_type = Some(content_type.to_string());
        self
    }

    fn body(mut self, body: &str) -> Self {
        self.body = Some(String::from(body));
        self
    }

    fn build(self) -> Result<HttpResponse, &'static str> {
        if self.status_code.is_none() {
            return Err("Status code must be provided.");
        }

        let body = self.body.unwrap_or_default();

        let mut headers: Vec<HttpHeader> = Vec::new();

        if !body.is_empty() {
            headers.push(HttpHeader::new(
                "Content-Type",
                self.content_type
                    .unwrap_or(String::from("text/plain"))
                    .as_str(),
            ));

            headers.push(HttpHeader::new(
                "Content-Length",
                body.len().to_string().as_str(),
            ));
        }

        Ok(HttpResponse {
            version: self.http_version.unwrap_or("HTTP/1.1".to_string()),
            status_code: self.status_code.unwrap().status(),
            headers,
            body,
        })
    }
}
