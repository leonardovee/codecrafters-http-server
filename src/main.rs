mod http_response;
mod prefix_tree;

use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

use http_response::{HttpResponse, HttpStatus};
use prefix_tree::PrefixTree;

fn main() {
    let mut ptree = PrefixTree::new();
    ptree.insert("/", "GET", |_| "hello".to_string());
    ptree.insert("/echo/{text}", "GET", |params| {
        params.get("text").unwrap().to_string()
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

    match ptree.search(path) {
        Some((route_method, handler, params)) if route_method == method => {
            let body = handler(&params);
            send_response(
                &mut stream,
                HttpResponse::new(HttpStatus::Ok)
                    .with_body(body)
                    .to_string(),
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
