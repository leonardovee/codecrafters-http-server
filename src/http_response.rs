use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum HttpStatus {
    Ok = 200,
    BadRequest = 400,
    NotFound = 404,
    InternalServerError = 500,
}

impl HttpStatus {
    fn as_str(&self) -> &'static str {
        match self {
            HttpStatus::Ok => "OK",
            HttpStatus::BadRequest => "Bad Request",
            HttpStatus::NotFound => "Not Found",
            HttpStatus::InternalServerError => "InternalServerError",
        }
    }
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    status_code: HttpStatus,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl HttpResponse {
    pub fn new(status_code: HttpStatus) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "text/plain".to_string());

        HttpResponse {
            status_code,
            headers,
            body: Vec::new(),
        }
    }

    pub fn with_body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self.headers
            .insert("Content-Length".to_string(), self.body.len().to_string());
        self
    }

    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn to_string(&self) -> String {
        let status_line = format!(
            "HTTP/1.1 {} {}\r\n",
            self.status_code as u16,
            self.status_code.as_str()
        );
        let headers: String = self
            .headers
            .iter()
            .map(|(k, v)| format!("{}: {}\r\n", k, v))
            .collect();
        let body = String::from_utf8_lossy(&self.body);

        format!("{}{}\r\n{}", status_line, headers, body)
    }
}
