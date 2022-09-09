use std::path::{Path};
use crate::ircore::doc::Document;
use std::io::{self};
use super::dir::{DirIter, ParseString};
use crate::ircore::doc::cfg::Cfg;

pub struct TextDocParser {
}

impl TextDocParser {
    pub fn docs<'a>(path: &str, cfg: &'a Cfg) -> DirIter<'a> {
        DirIter::new(path, Self::parse_string, cfg)
    }
}

impl ParseString for TextDocParser {
    fn parse_string(path: &Path, text: &str, _cfg: &Cfg) -> io::Result<Document> {
        let path_string = path.to_string_lossy().to_string();
        Ok(Document::new(text.to_string(), path_string))
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_text() {
        if let Ok(doc) = TextDocParser::parse_string(
                Path::new("./sample_corpus/romeo_juliet/a/1.txt"),
                "Do you quarrel, sir?",
                &Cfg::new()){
            assert_eq!(doc.get_content(), "Do you quarrel, sir?");
            assert_eq!(doc.get_path(), "./sample_corpus/romeo_juliet/a/1.txt");
        }else{
            assert!(false);
        }
    }

    #[test]
    fn test_txt_file_parser_docs() {
        let docs:Vec<Document> = TextDocParser::docs("./sample_corpus/romeo_juliet",
                                &Cfg::new()).collect();
        assert_eq!(docs.len(), 5);
    }
}