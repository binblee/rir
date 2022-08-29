use std::fs::{self};
use std::path::{Path, PathBuf};
use super::Document;
use encoding_rs::{ISO_8859_2};
use encoding_rs_io::{DecodeReaderBytesBuilder};
use std::io::Read;
use std::fs::File;
use std::io::{self};
use std::io::ErrorKind::{self};
use std::collections::VecDeque;

pub struct TextFileParser {
}

impl TextFileParser {
    pub fn docs(path: &str) -> DirIter {
        DirIter {
            path_queue: VecDeque::from(vec!(PathBuf::from(path))),
        }
    }
    fn parse_file(path: &Path) -> io::Result<Document> {
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

pub struct DirIter {
    path_queue: VecDeque<PathBuf>,
}

impl Iterator for DirIter {
    type Item = Document;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(path) = self.path_queue.pop_front(){
            if path.is_file(){
                match TextFileParser::parse_file(&path){
                    Ok(doc) => return Some(doc),
                    Err(e) => {
                        println!("{}", e);
                    }
                }
            }else if path.is_dir(){
                // println!("indexing {}...", path.display());
                for entry_result in path.read_dir().expect(&format!("read dir {} failed", path.display())) {
                    if let Ok(entry) = entry_result{
                        let entry_path = entry.path();
                        self.path_queue.push_back(entry_path)   
                    }
                }
            }
            return self.next();    
        }
        return None;
    }
}

#[test]
fn test_plain_text() {
    if let Ok(doc) = TextFileParser::parse_file(Path::new("./sample_corpus/romeo_juliet/a/1.txt")){
        assert_eq!(doc.get_content(), "Do you quarrel, sir?");
        assert_eq!(doc.get_path(), "./sample_corpus/romeo_juliet/a/1.txt");
    }else{
        assert!(false);
    }
    if let Err(e) = TextFileParser::parse_file(Path::new("./sample_corpus/romeo_juliet/non-exist.txt")){
        assert_eq!(e.kind(), io::ErrorKind::Other);
    }
}

#[test]
fn test_load_file_encoding_iso8859() {
    let filename = "/Users/libin/Code/github.com/binblee/sir/20news-18828/comp.windows.x/67305";
    if let Ok(doc) = TextFileParser::parse_file(Path::new(filename)){
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
    let docs:Vec<Document> = TextFileParser::docs("./sample_corpus/romeo_juliet").collect();
    assert_eq!(docs.len(), 5);
}