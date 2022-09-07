use std::fs::{self};
use std::path::{Path};
use crate::ircore::doc::Document;
use encoding_rs::{ISO_8859_2};
use encoding_rs_io::{DecodeReaderBytesBuilder};
use std::io::Read;
use std::fs::File;
use std::io::{self};
use std::io::ErrorKind;
use super::dir::{DirIter, ParseFile};
use crate::ircore::doc::cfg::Cfg;

pub struct TextFileParser {
}

impl TextFileParser {
    pub fn docs<'a>(path: &str, cfg: &'a Cfg) -> DirIter<'a> {
        DirIter::new(path, Self::parse_file, cfg)
    }
    fn read_to_string_non_utf8_encoding(path: &Path) -> io::Result<String> {
        let mut dest = String::new();
        match File::open(path){
            Ok(source_file) => {
                let mut decoder = DecodeReaderBytesBuilder::new()
                .encoding(Some(ISO_8859_2))
                .build(source_file);
    
                match decoder.read_to_string(&mut dest){
                    Ok(res) => {
                        assert!(res > 0);
                        return Ok(dest);    
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }    
            },
            Err(e) => return Err(e),
        }
    }
}

impl ParseFile for TextFileParser {
    fn parse_file(path: &Path, _cfg: &Cfg) -> io::Result<Document> {
        let path_string = path.to_string_lossy().to_string();
        if path.is_file() {
            match fs::read_to_string(path) {
                Ok(c) => return Ok(Document::new(c, path_string)),
                Err(e) => {
                    if let Some(filename) = path.file_name(){
                        //by default, ignore hidden files on unix like platforms
                        if filename.to_string_lossy().to_string().starts_with("."){
                            return Err(io::Error::new(ErrorKind::Other, format!("ignore {}, as it is a hidden file", path_string)))
                        }
                    }
                    if e.kind() == ErrorKind::InvalidData {
                        match Self::read_to_string_non_utf8_encoding(path) {
                            Ok(c) => return Ok(Document::new(c, path_string)),
                            Err(e) => return Err(e),
                        }
                    }else{
                        return Err(e);
                    }
                }
            }
        }else{
            return Err(io::Error::new(ErrorKind::Other, format!("{} is not a file", path_string)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_text() {
        if let Ok(doc) = TextFileParser::parse_file(Path::new("./sample_corpus/romeo_juliet/a/1.txt"),
                                                    &Cfg::new()){
            assert_eq!(doc.get_content(), "Do you quarrel, sir?");
            assert_eq!(doc.get_path(), "./sample_corpus/romeo_juliet/a/1.txt");
        }else{
            assert!(false);
        }
        if let Err(e) = TextFileParser::parse_file(Path::new("./sample_corpus/romeo_juliet/non-exist.txt"),
                                                    &Cfg::new()){
            assert_eq!(e.kind(), io::ErrorKind::Other);
        }
    }

    #[test]
    fn test_load_file_encoding_iso8859() {
        let filename = "/Users/libin/Code/github.com/binblee/sir/20news-18828/comp.windows.x/67305";
        if let Ok(doc) = TextFileParser::parse_file(Path::new(filename), &Cfg::new()){
            assert_eq!(doc.get_path(), filename);
        }
    }

    #[test]
    fn test_load_non_utf8_file1(){
        match TextFileParser::read_to_string_non_utf8_encoding(Path::new("sample_corpus/non_utf8_encoding/103700")){
            Ok(content) => assert!(content.len() > 0),
            Err(_error) => assert!(false),
        }
    }

    #[test]
    fn test_load_non_utf8_file2(){
        match TextFileParser::read_to_string_non_utf8_encoding(Path::new("sample_corpus/non_utf8_encoding/67305")){
            Ok(content) => assert!(content.len() > 0),
            Err(_error) => assert!(false),
        }
    }

    #[test]
    fn test_encoding_rs_io() {
        use encoding_rs_io::DecodeReaderBytes;
        let source_data = &b"\xFF\xFEf\x00o\x00o\x00b\x00a\x00r\x00"[..];
        // N.B. `source_data` can be any arbitrary io::Read implementation.
        let mut decoder = DecodeReaderBytes::new(source_data);

        let mut dest = String::new();
        // decoder implements the io::Read trait, so it can easily be plugged
        // into any consumer expecting an arbitrary reader.
        let result;
        if let Ok(res) = decoder.read_to_string(&mut dest){
            assert_eq!(dest, "foobar");
            result = res;
            assert!(result > 0);
        }else{
            assert!(false);
        }
    }

    #[test]
    fn test_txt_file_parser_docs() {
        let docs:Vec<Document> = TextFileParser::docs("./sample_corpus/romeo_juliet",
                                &Cfg::new()).collect();
        assert_eq!(docs.len(), 5);
    }
}