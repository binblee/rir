use std::path::Path;
use crate::ircore::doc::dir::DirIter;
use crate::ircore::doc::json::JsonFileParser;
use crate::ircore::doc::text::TextFileParser;
use crate::ircore::common::CFG_NAME;
use crate::ircore::doc::cfg::Cfg;
pub struct DocParser {
    path: String,
    cfg: Cfg,
}
use std::fs;

impl DocParser {
    pub fn new(path: &str) -> Self {
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
    pub fn docs(&self) -> DirIter {
        if self.cfg.is_json() {
            return JsonFileParser::docs(&self.path, &self.cfg);
        }else{
            return TextFileParser::docs(&self.path, &self.cfg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_json() {
        let mut dp = DocParser::new("./sample_corpus/wiki_zh");
        assert_eq!(dp.cfg.is_json(), true);
        dp = DocParser::new("./sample_corpus/romeo_juliet");
        assert_eq!(dp.cfg.is_json(), false);
    }
}