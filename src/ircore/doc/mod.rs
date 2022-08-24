pub mod text;

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
