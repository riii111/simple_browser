use alloc::string::{String, ToString};

#[derive(Debug, Clone, PartialEq)]
pub struct Url {
    url: String,
    host: String,
    port: String,
    path: String,
    searchPart: String,
}

impl Url {
    pub fn new(url: String) -> Self {
        Self {
            url,
            host: "".to_string(),
            port: "".to_string(),
            path: "".to_string(),
            searchPart: "".to_string(),
        }
    }
}
