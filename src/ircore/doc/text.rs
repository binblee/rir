use std::path::{Path};
use crate::ircore::doc::Document;
use std::io::{self};
use super::dir::{DirIter, ParseString};
use crate::ircore::doc::cfg::Cfg;

pub struct TextDocParser {
}

impl TextDocParser {
    pub fn docs<'a>(path: &str, cfg: &'a Cfg) -> DirIter<'a> {
        log::info!("Parse text files.");
        DirIter::new(path, Self::parse_string, cfg)
    }
}

impl ParseString for TextDocParser {
    fn parse_string(path: &Path, text: &str, _cfg: &Cfg) -> io::Result<Vec<Document>> {
        let path_string = path.to_string_lossy().to_string();
        Ok(vec![Document::new(text.to_string(), path_string)])
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ircore::doc::doc_parser::DocParser;

    #[test]
    fn test_plain_text() {
        if let Ok(doc) = TextDocParser::parse_string(
                Path::new("./sample_corpus/romeo_juliet/a/1.txt"),
                "Do you quarrel, sir?",
                &Cfg::new()){
            assert_eq!(doc[0].get_content(), "Do you quarrel, sir?");
            assert_eq!(doc[0].get_path(), "./sample_corpus/romeo_juliet/a/1.txt");
        }else{
            assert!(false);
        }
    }

    #[test]
    fn test_txt_file_parser_docs() {
        let docs:Vec<Vec<Document>> = TextDocParser::docs("./sample_corpus/romeo_juliet",
                                &Cfg::new()).collect();
        assert_eq!(docs.len(), 5);
    }

    #[test]
    fn test_load_english_text() {
        let dp = DocParser::new("./sample_corpus/romeo_juliet");
        assert_eq!(dp.get_config().get_file_type(), "text");
        let docs:Vec<Vec<Document>> = dp.docs().collect();
        // only text(utf-8) files are loaded, binary files are ignored.
        assert_eq!(docs.len(), 5); 
        for doc in docs {
            assert_eq!(doc.len(), 1);
        }
    }

    #[test]
    fn test_load_chinese_text() {
        let dp = DocParser::new("./sample_corpus/sanguo");
        assert_eq!(dp.get_config().get_file_type(), "text");
        let docs:Vec<Vec<Document>> = dp.docs().collect();
        assert_eq!(docs.len(), 19);
        for doc in docs {
            assert_eq!(doc.len(), 1);
        }
    }
}