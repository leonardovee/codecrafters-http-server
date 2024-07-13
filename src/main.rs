mod http_request;
mod http_response;
mod prefix_tree;

use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
};

use http_request::HttpRequest;
use http_response::{HttpResponse, HttpStatus};
use prefix_tree::PrefixTree;

fn main() {
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

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream, &ptree);
                println!("accepted new connection");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream, ptree: &PrefixTree) {
    let mut reader = BufReader::new(&mut stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line).unwrap();

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        send_response(
            &mut stream,
            HttpResponse::new(HttpStatus::BadRequest).to_string(),
        );
        return;
    }

    let method = parts[0];
    let path = parts[1];

    let mut headers = HashMap::new();
    for line in reader.by_ref().lines() {
        let line = line.unwrap();
        if line.is_empty() {
            break;
        }
        let parts: Vec<&str> = line.splitn(2, ": ").collect();
        if parts.len() == 2 {
            headers.insert(parts[0].to_lowercase(), parts[1].to_string());
        }
    }

    let mut body = Vec::new();
    if let Some(length) = headers.get("content-length") {
        let length: usize = length.parse().unwrap_or(0);
        reader.take(length as u64).read_to_end(&mut body).unwrap();
    }

    match ptree.search(path) {
        Some((route_method, handler, params)) if route_method == method => {
            send_response(
                &mut stream,
                handler(HttpRequest::new(headers, params, body)).to_string(),
            );
        }
        Some(_) => send_response(
            &mut stream,
            HttpResponse::new(HttpStatus::BadRequest).to_string(),
        ),
        None => send_response(
            &mut stream,
            HttpResponse::new(HttpStatus::NotFound).to_string(),
        ),
    }
}

fn send_response(stream: &mut TcpStream, response: String) {
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
