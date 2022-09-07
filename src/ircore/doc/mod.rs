pub mod dir;
pub mod text;
pub mod json;
pub mod doc_parser;
pub mod cfg;


#[derive(PartialEq, Debug)]
pub struct Document {
    content: String,
    path: String,
}

impl Document {
    pub fn new(content: String, path: String) -> Self {
        Document {
            content: content.to_string(),
            path: path.to_string(),
        }
    }
    pub fn get_content(&self) -> &str {
        &self.content
    }
    pub fn get_path(&self) -> &str {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document() {
        let doc = Document::new("content: String".to_string(), "path: String".to_string());
        assert_eq!(doc.get_content(), "content: String");
        assert_eq!(doc.get_path(), "path: String");
    }
}