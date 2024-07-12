#[derive(Debug)]
pub struct HttpHeader {
    pub name: String,
    pub value: String,
}

impl HttpHeader {
    pub fn new(name: &str, value: &str) -> HttpHeader {
        HttpHeader {
            name: String::from(name),
            value: String::from(value),
        }
    }

    pub fn output(self) -> String {
        format!("{}: {}", self.name, self.value)
    }
}
