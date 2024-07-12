use super::http_header::HttpHeader;

#[derive(Debug)]
pub struct HttpResponse {
    pub version: String,
    pub status_code: HttpStatus,
    pub headers: Vec<HttpHeader>,
    pub body: Vec<u8>,
}

#[derive(Debug)]
pub struct HttpStatus {
    code: u16,
    description: &'static str,
}

pub enum HttpStatusCode {
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

impl HttpResponse {
    pub fn not_found() -> HttpResponse {
        HttpResponseBuilder::new(HttpStatusCode::NotFound)
            .build()
            .unwrap()
    }

    pub fn output(self) -> Vec<u8> {
        let mut response_lines: Vec<String> = vec![format!(
            "{} {} {}",
            self.version, self.status_code.code, self.status_code.description
        )];

        for header in self.headers {
            response_lines.push(header.output())
        }

        let mut response_str = response_lines.join("\r\n");
        response_str.push_str("\r\n\r\n");

        let mut buffer: Vec<u8> = Vec::from(response_str.as_bytes());
        buffer.extend(self.body);

        println!("RESPONSE:\r\n{}", response_str);
        buffer
    }
}

pub struct HttpResponseBuilder {
    status_code: HttpStatusCode,
    http_version: Option<String>,
    content_type: Option<String>,
    content_encoding: Option<String>,
    body: Option<Vec<u8>>,
}

impl HttpResponseBuilder {
    pub fn new(http_status_code: HttpStatusCode) -> Self {
        HttpResponseBuilder {
            status_code: http_status_code,
            http_version: None,
            content_type: None,
            content_encoding: None,
            body: None,
        }
    }

    pub fn content_type(mut self, content_type: &str) -> Self {
        self.content_type = Some(content_type.to_string());
        self
    }

    pub fn content_encoding(mut self, content_encoding: &str) -> Self {
        self.content_encoding = Some(content_encoding.to_string());
        self
    }

    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }

    pub fn build(self) -> Result<HttpResponse, &'static str> {
        let body = self.body.unwrap_or_default();

        let mut headers: Vec<HttpHeader> = Vec::new();

        if !body.is_empty() {
            if self.content_encoding.is_some() {
                headers.push(HttpHeader::new(
                    "Content-Encoding",
                    self.content_encoding.unwrap().as_str(),
                ));
            }

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
            status_code: self.status_code.status(),
            version: self.http_version.unwrap_or("HTTP/1.1".to_string()),
            headers,
            body,
        })
    }
}
