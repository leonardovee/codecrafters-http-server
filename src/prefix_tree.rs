use std::{collections::HashMap, future::Future, pin::Pin};

use crate::{http_request::HttpRequest, http_response::HttpResponse};

type AsyncHandler =
    Box<dyn Fn(HttpRequest) -> Pin<Box<dyn Future<Output = HttpResponse> + Send>> + Send + Sync>;

struct Node {
    children: HashMap<String, Node>,
    handlers: HashMap<String, AsyncHandler>,
}

impl Node {
    fn new() -> Self {
        Node {
            children: HashMap::new(),
            handlers: HashMap::new(),
        }
    }
}

pub struct PrefixTree {
    root: Node,
}

impl PrefixTree {
    pub fn new() -> Self {
        PrefixTree { root: Node::new() }
    }

    pub fn insert<F, Fut>(&mut self, path: &str, method: &str, handler: F)
    where
        F: Fn(HttpRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        let mut current = &mut self.root;
        for part in path.split('/').filter(|&x| !x.is_empty()) {
            current = current
                .children
                .entry(part.to_string())
                .or_insert_with(Node::new);
        }
        current.handlers.insert(
            method.to_string(),
            Box::new(move |req| Box::pin(handler(req))),
        );
    }

    pub fn search(
        &self,
        path: &str,
        method: &str,
    ) -> Option<(&AsyncHandler, HashMap<String, String>)> {
        let mut current = &self.root;
        let mut params = HashMap::new();
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        for part in parts {
            if let Some(child) = current.children.get(part) {
                current = child;
            } else {
                let param_node = current
                    .children
                    .iter()
                    .find(|(k, _)| k.starts_with('{') && k.ends_with('}'));
                if let Some((param_name, child)) = param_node {
                    let param_name = &param_name[1..param_name.len() - 1];
                    params.insert(param_name.to_string(), part.to_string());
                    current = child;
                } else {
                    return None;
                }
            }
        }

        current.handlers.get(method).map(|h| (h, params))
    }
}
