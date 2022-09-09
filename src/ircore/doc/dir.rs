use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use super::Document;
use crate::ircore::doc::cfg::Cfg;
use std::io::{self, Read, ErrorKind};
use std::fs::{self, File};
use encoding_rs::{ISO_8859_2};
use encoding_rs_io::{DecodeReaderBytesBuilder};

type FnParseString = fn(&Path, &str, &Cfg) -> io::Result<Document>;

pub trait ParseString {
    fn parse_string(path: &Path, text: &str, cfg: &Cfg) -> io::Result<Document>;
}

pub struct DirIter<'a> {
    path_queue: VecDeque<PathBuf>,
    fn_parsestring: FnParseString,
    cfg: &'a Cfg,
}

impl<'a> DirIter<'a> {
    pub fn new(path: &str, 
            fn_parsestring: FnParseString,
            cfg: &'a Cfg) -> Self{
        DirIter {
            path_queue: VecDeque::from(vec!(PathBuf::from(path))),
            fn_parsestring: fn_parsestring,
            cfg: cfg,
        }
    }
    fn ignore(path: &Path) -> bool {
        if let Some(filename) = path.file_name(){
            //by default, ignore hidden files on unix like platforms
            if filename.to_string_lossy().to_string().starts_with("."){
                return true;
            }
        }
        return false;
    }
    fn load_content(path: &Path) -> io::Result<String> {
        match fs::read_to_string(path){
            Ok(c) => return Ok(c),
            Err(e) => {
                if e.kind() == ErrorKind::InvalidData {
                    match Self::read_to_string_non_utf8_encoding(path) {
                        Ok(c) => return Ok(c),
                        Err(e) => return Err(e),
                    }
                }else{
                    return Err(e);
                }
            }
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

impl<'a> Iterator for DirIter<'a> {
    type Item = Document;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(path) = self.path_queue.pop_front(){
            if path.is_file() {
                if Self::ignore(&path) {
                    let path_string = path.to_string_lossy().to_string();
                    println!("ignore {}", path_string);
                }else{
                    match Self::load_content(&path){
                        Ok(c) => match (self.fn_parsestring)(&path, &c, &self.cfg){
                            Ok(doc) => return Some(doc),
                            Err(e) => {
                                println!("{}", e);
                            }
                        },
                        Err(e) => {
                            println!("{}", e);
                        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_non_utf8_file1(){
        match DirIter::read_to_string_non_utf8_encoding(Path::new("sample_corpus/non_utf8_encoding/103700")){
            Ok(content) => assert!(content.len() > 0),
            Err(_error) => assert!(false),
        }
    }

    #[test]
    fn test_load_non_utf8_file2(){
        match DirIter::read_to_string_non_utf8_encoding(Path::new("sample_corpus/non_utf8_encoding/67305")){
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


}