use itertools::Itertools;

use super::http_header::HttpHeader;

#[derive(Debug)]
pub struct HttpRequest {
    pub request_type: HttpRequestType,
    pub request_path: String,
    pub headers: Vec<HttpHeader>,
    pub body: Option<String>,
}

#[derive(Debug)]
pub enum HttpRequestType {
    Get,
    Put,
    Post,
    Delete,
    Patch,
}

impl HttpRequest {
    pub fn from_str(request_str: &str) -> HttpRequest {
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
