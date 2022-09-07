use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use super::Document;
use crate::ircore::doc::cfg::Cfg;
use std::io;

type FnParseFile = fn(&Path, &Cfg) -> io::Result<Document>;

pub trait ParseFile {
    fn parse_file(path: &Path, cfg: &Cfg) -> io::Result<Document>;
}

pub struct DirIter<'a> {
    path_queue: VecDeque<PathBuf>,
    fn_parsefile: FnParseFile,
    cfg: &'a Cfg,
}

impl<'a> DirIter<'a> {
    pub fn new(path: &str, fn_parsefile: FnParseFile, cfg: &'a Cfg) -> Self{
        DirIter {
            path_queue: VecDeque::from(vec!(PathBuf::from(path))),
            fn_parsefile: fn_parsefile,
            cfg: cfg,
        }
    }
}

impl<'a> Iterator for DirIter<'a> {
    type Item = Document;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(path) = self.path_queue.pop_front(){
            if path.is_file(){
                match (self.fn_parsefile)(&path, &self.cfg){
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