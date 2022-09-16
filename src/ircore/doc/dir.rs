use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use super::Document;
use crate::ircore::doc::cfg::Cfg;
use std::io;
use std::fs;

pub type FnParseString = fn(&Path, &str, &Cfg) -> io::Result<Vec<Document>>;

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
        fs::read_to_string(path)
    }
}

impl<'a> Iterator for DirIter<'a> {
    type Item = Vec<Document>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(path) = self.path_queue.pop_front(){
            let path_string = path.to_string_lossy().to_string();
            if Self::ignore(&path){
                log::info!("ignore {}", path_string);
            }else if path.is_file() {
                match Self::load_content(&path){
                    Ok(c) => match (self.fn_parsestring)(&path, &c, &self.cfg){
                        Ok(docs) => return Some(docs),
                        Err(e) => {
                            log::error!("{}: {}", path_string, e);
                        }
                    },
                    Err(e) => {
                        log::error!("{}: {}", path_string, e);
                    }
                }
            }else if path.is_dir(){
                log::debug!("{}...", path.display());
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
    fn test_ignore() {
        assert!(!DirIter::ignore(Path::new("./sample_corpus")));
        assert!(DirIter::ignore(Path::new("./sample_corpus/.rircfg")));
        assert!(!DirIter::ignore(Path::new(".")));
        assert!(DirIter::ignore(Path::new("./.rir")));
    }

}