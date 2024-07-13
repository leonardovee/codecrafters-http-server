use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
mod http_request;
mod http_response;
mod prefix_tree;

use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;

use std::collections::HashMap;
use std::sync::Arc;

use http_request::HttpRequest;
use http_response::{HttpResponse, HttpStatus};
use prefix_tree::PrefixTree;

#[tokio::main]
async fn main() {
    let mut ptree = PrefixTree::new();
    ptree.insert("/", "GET", |_| {
        HttpResponse::new(HttpStatus::Ok).with_body("hello".to_string())
    });
    ptree.insert("/echo/{text}", "GET", |req| {
        HttpResponse::new(HttpStatus::Ok).with_body(req.params.get("text").unwrap().to_string())
    });
    ptree.insert("/user-agent", "GET", |req| {
        HttpResponse::new(HttpStatus::Ok)
            .with_body(req.headers.get("user-agent").unwrap().to_string())
    });
    let ptree = Arc::new(ptree);

    let listener = TcpListener::bind("127.0.0.1:4221").await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let ptree_clone = Arc::clone(&ptree);
        println!("Connection accepted");
        tokio::spawn(async move {
            handle_connection(socket, &ptree_clone).await;
        });
    }
}

async fn handle_connection(mut stream: TcpStream, ptree: &PrefixTree) {
    let mut reader = BufReader::new(&mut stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line).await.unwrap();

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        send_response(
            &mut stream,
            HttpResponse::new(HttpStatus::BadRequest).to_string(),
        )
        .await;
        return;
    }

    let method = parts[0];
    let path = parts[1];

    let mut headers = HashMap::new();
    loop {
        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line).await.unwrap();
        if bytes_read == 0 || line == "\r\n" {
            break;
        }
        let line = line.trim();
        if let Some((key, value)) = line.split_once(": ") {
            headers.insert(key.to_lowercase(), value.to_string());
        }
    }

    let mut body = Vec::new();
    if let Some(length) = headers.get("content-length") {
        let length: usize = length.parse().unwrap_or(0);
        reader
            .take(length as u64)
            .read_to_end(&mut body)
            .await
            .unwrap();
    }

    match ptree.search(path) {
        Some((route_method, handler, params)) if route_method == method => {
            send_response(
                &mut stream,
                handler(HttpRequest::new(headers, params, body)).to_string(),
            )
            .await;
        }
        Some(_) => {
            send_response(
                &mut stream,
                HttpResponse::new(HttpStatus::BadRequest).to_string(),
            )
            .await
        }
        None => {
            send_response(
                &mut stream,
                HttpResponse::new(HttpStatus::NotFound).to_string(),
            )
            .await
        }
    }
}

async fn send_response(stream: &mut TcpStream, response: String) {
    stream.write_all(response.as_bytes()).await.unwrap();
    stream.flush().await.unwrap();
}
