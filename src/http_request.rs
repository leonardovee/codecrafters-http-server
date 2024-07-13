use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub headers: HashMap<String, String>,
    pub params: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl HttpRequest {
    pub fn new(
        headers: HashMap<String, String>,
        params: HashMap<String, String>,
        body: Vec<u8>,
    ) -> Self {
        HttpRequest {
            headers,
            params,
            body,
        }
    }
}
