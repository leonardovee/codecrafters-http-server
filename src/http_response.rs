use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum HttpStatus {
    Ok = 200,
    Created = 201,
    BadRequest = 400,
    NotFound = 404,
    InternalServerError = 500,
}

impl HttpStatus {
    fn as_str(&self) -> &'static str {
        match self {
            HttpStatus::Ok => "OK",
            HttpStatus::Created => "Created",
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
    pub body: Vec<u8>,
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

    pub fn to_string(&self) -> Vec<u8> {
        let mut response = Vec::new();

        response.extend_from_slice(
            format!(
                "HTTP/1.1 {} {}\r\n",
                self.status_code as u16,
                self.status_code.as_str()
            )
            .as_bytes(),
        );

        for (k, v) in &self.headers {
            response.extend_from_slice(format!("{}: {}\r\n", k, v).as_bytes());
        }

        response.extend_from_slice(b"\r\n");

        response.extend_from_slice(&self.body);

        response
    }
}
