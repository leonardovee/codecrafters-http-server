use std::collections::HashMap;

#[derive(Debug)]
struct Node {
    children: HashMap<String, Node>,
    is_endpoint: bool,
    method: Option<String>,
    handler: Option<fn(&HashMap<String, String>) -> String>,
}

impl Node {
    fn new() -> Self {
        Node {
            children: HashMap::new(),
            is_endpoint: false,
            method: None,
            handler: None,
        }
    }
}

#[derive(Debug)]
pub struct PrefixTree {
    root: Node,
}

impl PrefixTree {
    pub fn new() -> Self {
        PrefixTree { root: Node::new() }
    }

    pub fn insert(
        &mut self,
        path: &str,
        method: &str,
        handler: fn(&HashMap<String, String>) -> String,
    ) {
        let mut node = &mut self.root;
        for part in path.split('/').filter(|&x| !x.is_empty()) {
            node = node.children.entry(part.to_string()).or_insert(Node::new());
        }
        node.is_endpoint = true;
        node.method = Some(method.to_string());
        node.handler = Some(handler);
    }

    pub fn search(
        &self,
        path: &str,
    ) -> Option<(
        &str,
        fn(&HashMap<String, String>) -> String,
        HashMap<String, String>,
    )> {
        let mut node = &self.root;
        let mut params = HashMap::new();
        for part in path.split('/').filter(|&x| !x.is_empty()) {
            match node.children.get(part) {
                Some(child) => node = child,
                None => {
                    let param_node = node
                        .children
                        .iter()
                        .find(|(k, _)| k.starts_with('{') && k.ends_with('}'));
                    match param_node {
                        Some((key, child)) => {
                            let param_name = key.trim_start_matches('{').trim_end_matches('}');
                            params.insert(param_name.to_string(), part.to_string());
                            node = child;
                        }
                        None => return None,
                    }
                }
            }
        }
        if node.is_endpoint {
            node.method
                .as_ref()
                .and_then(|m| node.handler.map(|h| (m.as_str(), h, params)))
        } else {
            None
        }
    }
}
