use super::index::{SchemaDependIndex, PositionList, DocId};
use std::fs;
use std::fs::File;
use std::path::Path;
use std::collections::{HashMap};
use super::tokenizer::{normalize, parse_tokens};
use serde::{Serialize, Deserialize};
use bincode;
use std::io;
use std::io::{Write, Read};

#[derive(Debug, Serialize, Deserialize)]
pub struct Engine {
    index: PositionList,
    doc_info: HashMap<DocId, String>,
}

impl Engine {
    pub fn new() -> Self {
        Engine{
            index: PositionList::new(),
            doc_info: HashMap::new(),
        }
    }

    pub fn doc_count(&self) -> usize {
        self.doc_info.len()
    }

    pub fn load_from(path: &Path) -> Self {
        let mut reader = File::open(path).expect("cannot open idx file.");
        let mut encoded:Vec<u8> = vec![];
        if let Ok(_) = reader.read_to_end(&mut encoded){
            let decoded: Engine = bincode::deserialize(&encoded[..]).unwrap();
            return decoded;
        }
        Self::new()
    }

    pub fn build_index(&mut self, path: &Path) -> Result<usize, ()> {
        if path.is_file(){
            if let Ok(content) = fs::read_to_string(path){
                let normalized_content = normalize(&content);
                let tokens = parse_tokens(&normalized_content);
                let id = self.index.build_from(tokens);
                if let Some(path_str) = path.to_str(){
                    self.doc_info.insert(id, path_str.to_owned());
                }
            }
        }else if path.is_dir(){
            println!("indexing {}...", path.display());
            for entry_result in path.read_dir().expect(&format!("read dir {} failed",path.display())) {
                if let Ok(entry) = entry_result{
                    let entry_path = entry.path();
                    self.build_index(&entry_path)?;    
                }
            }    
        }
        Ok(self.doc_info.len())    
    }

    pub fn save_to(&mut self, path: &Path) -> io::Result<()> {
        let encoded: Vec<u8> = bincode::serialize(self).unwrap();
        let mut writer = File::create(path)?;
        writer.write_all(&encoded)?;
        Ok(())
    }

    pub fn search_phrase(&self, phrase_str: &str) -> Vec<&String>{
        let phrase_normalized = normalize(&phrase_str);
        let phrase_tokens = parse_tokens(&phrase_normalized);
        let hits_list = self.index.search_phrase(phrase_tokens);
        let mut docs = vec![];
        for hits in hits_list {
            if let Some(docname) = self.doc_info.get(&hits.docid){
                docs.push(docname);
            }
        }
        docs
    }

}

#[test]
fn test_build_index() {
    let mut engine = Engine::new();
    let res = engine.build_index(&Path::new("./samples"));
    assert_eq!(res, Ok(5));
}

#[test]
fn test_find_phrase() {
    let mut engine = Engine::new();
    let res = engine.build_index(&Path::new("./samples"));
    assert_eq!(res, Ok(5));
    let mut docs = engine.search_phrase("Quarrel sir");
    use std::collections::HashSet;
    let doc_set: HashSet<&String> = HashSet::from_iter(docs);
    assert_eq!(doc_set, HashSet::from([&"./samples/a/1.txt".to_string(), &"./samples/a/2.txt".to_string()]));
    docs = engine.search_phrase("sir");
    assert_eq!(docs.len(), 4);

    docs = engine.search_phrase("non-exist");
    assert_eq!(docs.len(), 0);

    docs = engine.search_phrase("Sir non-exist");
    assert_eq!(docs.len(), 0);

    docs = engine.search_phrase("Sir");
    assert_eq!(docs.len(), 4);

}

#[test]
fn test_save_and_load_index() {
    let mut engine = Engine::new();
    let res = engine.build_index(&Path::new("./samples"));
    assert_eq!(res, Ok(5));
    assert_eq!(engine.doc_count(), 5);
    let index_path = &Path::new(".rir/samples.idx");
    let _ = engine.save_to(index_path);
    let loaded_engine = Engine::load_from(index_path);
    assert_eq!(loaded_engine.doc_count(), 5);
    let docs = engine.search_phrase("Quarrel sir");
    use std::collections::HashSet;
    let doc_set: HashSet<&String> = HashSet::from_iter(docs);
    assert_eq!(doc_set, HashSet::from([&"./samples/a/1.txt".to_string(), &"./samples/a/2.txt".to_string()]));
}