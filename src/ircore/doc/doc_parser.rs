use std::path::Path;
use std::fs;
use crate::ircore::doc::dir::DirIter;
use crate::ircore::doc::json::{self};
use crate::ircore::doc::text::{self};
use crate::ircore::doc::jsonlines::{self};
use crate::ircore::CFG_NAME;
use crate::ircore::doc::cfg::Cfg;
use std::collections::HashMap;
use crate::ircore::doc::dir::FnParseString;
use once_cell::sync::Lazy;
use std::sync::RwLock;
use std::sync::Once;

pub struct DocParser {
    path: String,
    cfg: Cfg,
}

static HANDLERS: Lazy<RwLock<HashMap<String, FnParseString>>> = Lazy::new(||{
        let m = HashMap::new();
        RwLock::new(m)
    }
);

static INIT: Once = Once::new();

impl DocParser {
    pub fn new(path: &str) -> Self {
        Self::init();
        if Path::new(path).is_dir() {
            let dir_path = Path::new(path);
            let cfg_path = dir_path.join(Path::new(CFG_NAME));
            if let Ok(cfg_str) = fs::read_to_string(cfg_path){
                match serde_yaml::from_str(&cfg_str){
                    Ok(cfg) => return DocParser {
                        path: String::from(path),
                        cfg: cfg,
                    },
                    _ => (),
                }    
            }
        }
        return DocParser{
            path: String::from(path),
            cfg: Cfg::new(),
        };

    }
    pub fn init(){
        INIT.call_once(||{
            Self::register(text::FILETYPE, text::parse_text);
            Self::register(json::FILETYPE, json::parse_json);
            Self::register(jsonlines::FILETYPE, jsonlines::parse_jsonlines);    
        });
    }
    pub fn get_config(&self) -> &Cfg {
        &self.cfg
    }

    pub fn register(filetype: &str, handler: FnParseString) {
        let mut handlers = HANDLERS.write().unwrap();
        handlers.insert(filetype.to_string(), handler);
    }

    pub fn docs(&self) -> DirIter {
        let handler;
        let handlers = HANDLERS.read().unwrap();
        let filetype = self.cfg.get_file_type();
        match handlers.get(filetype){
            Some(h) => handler = *h,
            None => handler = *handlers.get("text").unwrap(),
        }
        return DirIter::new(&self.path, handler, &self.cfg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config() {
        let mut dp = DocParser::new("./sample_corpus/wiki_zh");
        assert_eq!(dp.cfg.is_json(), true);
        dp = DocParser::new("./sample_corpus/romeo_juliet");
        assert_eq!(dp.cfg.is_json(), false);
        dp = DocParser::new("./sample_corpus/wiki_lines");
        assert_eq!(dp.cfg.is_jsonlines(), true);
    }

}