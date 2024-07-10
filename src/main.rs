use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                let request = read_request(&mut _stream).unwrap();

                let response: HttpResponse;
                if request.request_path == "/" {
                    response = HttpResponse::ok(None);
                } else if request.request_path.starts_with("/echo") {
                    let value = request.request_path.split('/').last().unwrap();
                    response = HttpResponse::ok(Some(value.to_string()));
                } else {
                    response = HttpResponse::not_found()
                }

                println!("{:#?}", request);
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
                Err(String::from("Reveived non-UTF8 data."))
            }
        }
        Err(e) => Err(format!("Failed to read bytes from stream: {}", e)),
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
        let first_line = request_str.split('\n').next().unwrap();
        let mut values = first_line.split(' ');
        HttpRequest {
            request_type: HttpRequestType::from_str(values.next().unwrap()).unwrap(),
            request_path: values.next().unwrap().to_string(),
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
    body: Option<String>,
}

struct HttpStatusCode {
    status_code: u16,
    description: String,
}

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
        let content_length: usize;

        if let Some(b) = &body {
            content_length = b.len();
        } else {
            content_length = 0;
        }

        let mut headers: Vec<HttpHeader> = vec![HttpHeader::new("Content-Type", "text/plain")];
        headers.push(HttpHeader::new(
            "Content-Length",
            content_length.to_string().as_str(),
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
            body: None,
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

        if self.body.is_some() {
            response_str.push_str(self.body.unwrap().as_str());
        }

        println!("{}", response_str);
        response_str.as_bytes().to_vec()
    }
}
