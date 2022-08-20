use super::index::{SchemaDependIndex, PositionList, DocId, DocScore, IndexSummary, TermId};
use super::analyzer::{Analyzer, AnalyzerSummary};
use std::fs::{self, File};
use std::path::Path;
use std::collections::{HashMap};
use serde::{Serialize, Deserialize};
use bincode;
use std::io::{self, Write, Read};

#[derive(Debug, Serialize, Deserialize)]
pub struct Engine {
    index: PositionList,
    analyzer: Analyzer,
    doc_info: HashMap<DocId, String>,
}

pub struct Summary {
    pub index: IndexSummary,
    pub analyzer: AnalyzerSummary,
}

impl Engine {
    pub fn new() -> Self {
        Engine{
            index: PositionList::new(),
            analyzer: Analyzer::new(),
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

    pub fn build_index_from(&mut self, path: &Path) -> Result<usize, ()> {
        if let Ok(doc_count) = self.build_index(path) {
            if let Ok(_) = self.compute_tf_idf() {
                return Ok(doc_count);
            }
        }
        Err(())
    }

    pub fn build_index(&mut self, path: &Path) -> Result<usize, ()> {
        if path.is_file(){
            if let Ok(content) = fs::read_to_string(path){
                let term_ids = self.analyzer.analyze(&content);
                let id = self.index.build_from(&term_ids);
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

    pub fn compute_tf_idf(&mut self) -> Result<(),()> {
        if self.doc_info.len() == 0 {
            return Err(());
        }
        self.index.compute_tf_idf()
    }

    pub fn save_to(&mut self, path: &Path) -> io::Result<()> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        let encoded: Vec<u8> = bincode::serialize(self).unwrap();
        let mut writer = File::create(path)?;
        writer.write_all(&encoded)?;
        Ok(())
    }

    pub fn summary(&self) -> Summary {
        Summary{
            index: self.index.summary(self.analyzer.get_dictionary()),
            analyzer: self.analyzer.summary(),
        }
    }

    fn query(&self, 
        phrase_str: &str,
        ignore_non_exist_term: bool,
        fn_query: fn(&PositionList, &Vec<TermId>) -> Vec<DocScore>) -> Vec<&String>{
        let (term_ids, unknown_terms) = self.analyzer.parse(phrase_str);
        let mut docs = vec![];
        if term_ids.len() == 0 {
            return docs;
        }
        if !ignore_non_exist_term && unknown_terms.len() > 0 {
            return docs;
        }
        let doc_scores = fn_query(&self.index, &term_ids);
        for doc in doc_scores {
            if let Some(docname) = self.doc_info.get(&doc.docid){
                docs.push(docname);
            }
        }    
        docs
    }

    pub fn search_phrase(&self, phrase_str: &str) -> Vec<&String>{
        self.query(phrase_str, false, PositionList::search_phrase)
    }

    pub fn rank_cosine(&self, query_str: &str) -> Vec<&String> {
        self.query(query_str, true, PositionList::rank_cosine)
    }

    pub fn rank_bm25(&self, query_str: &str) -> Vec<&String> {
        self.query(query_str, true, PositionList::rank_bm25)
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
    let res = engine.build_index_from(&Path::new("./samples"));
    assert_eq!(res, Ok(5));
    assert_eq!(engine.doc_count(), 5);
    let index_path = &Path::new(".rir/samples.idx");
    let _ = engine.save_to(index_path);
    let loaded_engine = Engine::load_from(index_path);
    assert_eq!(loaded_engine.doc_count(), 5);
    let docs = loaded_engine.search_phrase("Quarrel sir");
    use std::collections::HashSet;
    let doc_set: HashSet<&String> = HashSet::from_iter(docs);
    assert_eq!(doc_set, HashSet::from([&"./samples/a/1.txt".to_string(), &"./samples/a/2.txt".to_string()]));
}

#[test]
fn test_rank_cosine() {
    let mut engine = Engine::new();
    let res = engine.build_index_from(&Path::new("./samples"));
    assert_eq!(res, Ok(5));
    assert_eq!(engine.doc_count(), 5);
    let index_path = &Path::new(".rir/samples2.idx");
    let _ = engine.save_to(index_path);
    let loaded_engine = Engine::load_from(index_path);
    assert_eq!(loaded_engine.doc_count(), 5);
    let docs = loaded_engine.rank_cosine("Quarrel sir");
    use std::collections::HashSet;
    let doc_set: HashSet<&String> = HashSet::from_iter(docs);
    assert_eq!(doc_set, HashSet::from([
            &"./samples/a/2.txt".to_string(), 
            &"./samples/a/1.txt".to_string(),
            &"./samples/5.txt".to_string(), 
            &"./samples/b/3.txt".to_string(),
            ]));
}

#[test]
fn test_rank_bm25() {
    let mut engine = Engine::new();
    let res = engine.build_index_from(&Path::new("./samples"));
    assert_eq!(res, Ok(5));
    assert_eq!(engine.doc_count(), 5);
    let index_path = &Path::new(".rir/samples3.idx");
    let _ = engine.save_to(index_path);
    let loaded_engine = Engine::load_from(index_path);
    assert_eq!(loaded_engine.doc_count(), 5);
    let docs = loaded_engine.rank_bm25("Quarrel sir");
    use std::collections::HashSet;
    let doc_set: HashSet<&String> = HashSet::from_iter(docs);
    assert_eq!(doc_set, HashSet::from([
            &"./samples/a/2.txt".to_string(), 
            &"./samples/a/1.txt".to_string(),
            &"./samples/5.txt".to_string(), 
            &"./samples/b/3.txt".to_string(),
            ]));
}