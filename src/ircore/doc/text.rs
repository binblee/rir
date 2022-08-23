use std::fs::{self};
use std::path::Path;

pub struct PlainTextDoc {
    content: String,
    path: String,
    valid: bool,
}

impl PlainTextDoc {
    pub fn parse_file(path: &Path) -> Self {
        if path.is_file() {
            if let Ok(c) = fs::read_to_string(path) {
                return PlainTextDoc {
                    content: c,
                    path: path.to_string_lossy().to_string(),
                    valid: true,
                } 
            }
        }
        PlainTextDoc {
            content: String::default(),
            path: String::default(),
            valid: false,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }
}

use super::Document;
impl Document for PlainTextDoc {
    fn get_content(&self) -> &str {
        &self.content
    }

    fn get_path(&self) -> &str {
        &self.path
    }
}

#[test]
fn test_plain_text() {
    let mut doc = PlainTextDoc::parse_file(Path::new("./sample_corpus/a/1.txt"));
    assert!(doc.is_valid());
    assert_eq!(doc.get_content(), "Do you quarrel, sir?");
    doc = PlainTextDoc::parse_file(Path::new("./sample_corpus/non-exist.txt"));
    assert!(!doc.is_valid());
}

pub struct FileLoader {
}

impl FileLoader {
    pub fn load(path: &Path) -> PlainTextDoc {
        PlainTextDoc::parse_file(path)
    } 
}