use std::fs::{self};
use std::path::Path;
use super::super::document::Document;

pub trait TextFileLoader {
    fn parse_file(path: &Path) -> Option<Document>;
}

impl TextFileLoader for Document {
    fn parse_file(path: &Path) -> Option<Document> {
        if path.is_file() {
            if let Ok(c) = fs::read_to_string(path) {
                return Some(Document::new(c, path.to_string_lossy().to_string()));
            }
        }
        None
    }
}


#[test]
fn test_plain_text() {
    if let Some(doc) = Document::parse_file(Path::new("./sample_corpus/romeo_juliet/a/1.txt")){
        assert_eq!(doc.get_content(), "Do you quarrel, sir?");
        assert_eq!(doc.get_path(), "./sample_corpus/romeo_juliet/a/1.txt");
    }else{
        assert!(false);
    }
    let doc = Document::parse_file(Path::new("./sample_corpus/romeo_juliet/non-exist.txt"));
    assert_eq!(doc, None);
}

#[test]
fn test_load_file_encoding_iso8859() {
    let doc = Document::parse_file(Path::new("/Users/libin/Code/github.com/binblee/sir/20news-18828/comp.windows.x/67305"));
    assert_eq!(doc, None)
}