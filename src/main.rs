use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use itertools::Itertools;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                let request = read_request(&mut _stream).unwrap();
                println!("REQUEST:\r\n{:#?}", request);

                let response = generate_response(&request);
                write_response(_stream, response);
            }
            Err(e) => {
                eprintln!("error: {}", e);
            }
        }
    }
}

fn read_request(_stream: &mut TcpStream) -> Result<HttpRequest, String> {
    let mut read_buff = [0; 1024];
    match _stream.read(&mut read_buff) {
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

fn write_response(mut _stream: TcpStream, response: HttpResponse) {
    _stream
        .write_all(&response.output())
        .expect("Failed to write to stream");
}

#[derive(Debug)]
struct HttpRequest {
    request_type: HttpRequestType,
    request_path: String,
    headers: Vec<HttpHeader>,
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

        HttpRequest {
            request_type: HttpRequestType::from_str(values.next().unwrap()).unwrap(),
            request_path: values.next().unwrap().to_string(),
            headers,
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
    status_code: HttpStatusCode,
    headers: Vec<HttpHeader>,
    body: String,
}

struct HttpStatusCode {
    status_code: u16,
    description: String,
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
        let body = body.unwrap_or_default();

        let mut headers: Vec<HttpHeader> = vec![HttpHeader::new("Content-Type", "text/plain")];
        headers.push(HttpHeader::new(
            "Content-Length",
            body.len().to_string().as_str(),
        ));

        HttpResponse {
            version: String::from("HTTP/1.1"),
            status_code: HttpStatusCode {
                status_code: 200,
                description: String::from("OK"),
            },
            headers,
            body,
        }
    }

    fn not_found() -> HttpResponse {
        HttpResponse {
            version: String::from("HTTP/1.1"),
            status_code: HttpStatusCode {
                status_code: 404,
                description: String::from("Not Found"),
            },
            headers: Vec::new(),
            body: String::new(),
        }
    }

    fn output(self) -> Vec<u8> {
        let mut response_lines: Vec<String> = vec![format!(
            "{} {} {}",
            self.version, self.status_code.status_code, self.status_code.description
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
