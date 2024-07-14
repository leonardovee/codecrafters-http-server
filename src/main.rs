use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
mod http_request;
mod http_response;
mod prefix_tree;

use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;

use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::sync::Arc;

use http_request::HttpRequest;
use http_response::{HttpResponse, HttpStatus};
use prefix_tree::PrefixTree;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let files_dir: Arc<String> = Arc::new(args.last().unwrap_or(&"/tmp/".to_string()).to_string());

    let dir1 = files_dir.clone();
    let dir2 = files_dir.clone();

    let mut ptree = PrefixTree::new();
    ptree.insert("/", "GET", |_| async move {
        HttpResponse::new(HttpStatus::Ok).with_body("hello".to_string())
    });
    ptree.insert("/echo/{text}", "GET", |req| async move {
        HttpResponse::new(HttpStatus::Ok).with_body(req.params.get("text").unwrap().to_string())
    });
    ptree.insert("/user-agent", "GET", |req| async move {
        HttpResponse::new(HttpStatus::Ok)
            .with_body(req.headers.get("User-Agent").unwrap().to_string())
    });
    ptree.insert("/files/{file_name}", "POST", move |req| {
        let dir2_clone = Arc::clone(&dir2);
        async move {
            let file_name = req.params.get("file_name").unwrap();
            let file_path = Path::new(&*dir2_clone).join(file_name);

            match File::create(&file_path).await {
                Ok(mut file) => {
                    println!("{:?}", req);
                    if let Err(e) = file.write_all(&req.body).await {
                        HttpResponse::new(HttpStatus::InternalServerError)
                            .with_body(format!("Failed to write file: {}", e))
                    } else {
                        HttpResponse::new(HttpStatus::Created)
                            .with_body(format!("File '{}' created successfully", file_name))
                    }
                }
                Err(e) => HttpResponse::new(HttpStatus::InternalServerError)
                    .with_body(format!("Failed to create file: {}", e)),
            }
        }
    });
    ptree.insert("/files/{file_name}", "GET", move |req| {
        let dir1_clone = Arc::clone(&dir1);
        async move {
            let file_name = req.params.get("file_name").unwrap();
            let file_path = Path::new(&*dir1_clone).join(file_name);

            match File::open(&file_path).await {
                Ok(mut file) => {
                    let mut contents = Vec::new();
                    if let Err(e) = file.read_to_end(&mut contents).await {
                        return HttpResponse::new(HttpStatus::InternalServerError)
                            .with_body(format!("Failed to read file: {}", e));
                    }

                    HttpResponse::new(HttpStatus::Ok)
                        .with_body(contents)
                        .with_header("Content-Type", "application/octet-stream")
                        .with_header(
                            "Content-Disposition",
                            format!("attachment; filename=\"{}\"", file_name),
                        )
                }
                Err(_) => HttpResponse::new(HttpStatus::NotFound)
                    .with_body(format!("File not found: {}", file_name)),
            }
        }
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
            headers.insert(key.to_string(), value.to_string());
        }
    }

    let mut body = Vec::new();
    if let Some(length) = headers.get("Content-Length") {
        let length: usize = length.parse().unwrap_or(0);
        reader
            .take(length as u64)
            .read_to_end(&mut body)
            .await
            .unwrap();
    }
    println!("{:?}", body);

    match ptree.search(path, method) {
        Some((handler, params)) => {
            let mut response = handler(HttpRequest::new(headers.clone(), params, body)).await;
            if let Some(encoding) = headers.get("Accept-Encoding") {
                if encoding.contains("gzip") {
                    response = response.with_header("Content-Encoding", encoding);
                }
            }
            send_response(&mut stream, response.to_string()).await;
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
