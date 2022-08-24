use super::index::{SchemaDependIndex, PositionList, IndexStats};
use super::common::{DocId, RankingAlgorithm};
use super::analyzer::{Analyzer, AnalyzerStats};
use std::fs::{self, File};
use std::path::Path;
use std::collections::{HashMap};
use serde::{Serialize, Deserialize};
use bincode;
use std::io::{self, Write, Read};
use super::document::Document;
use super::doc::text::TextFileLoader;
use super::query::Query;

#[derive(Debug, Serialize, Deserialize)]
pub struct Engine {
    index: PositionList,
    analyzer: Analyzer,
    doc_meta: HashMap<DocId, String>,
}

pub struct Stats {
    pub index: IndexStats,
    pub analyzer: AnalyzerStats,
}

impl Engine {
    pub fn new() -> Self {
        Engine{
            index: PositionList::new(),
            analyzer: Analyzer::new(),
            doc_meta: HashMap::new(),
        }
    }

    pub fn doc_count(&self) -> usize {
        self.doc_meta.len()
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
            if let Some(doc) = Document::parse_file(path){
                let res = self.add_document(&doc);
                assert_eq!(res, Ok(()));
            }else{
                println!("failed to parse file {}", path.to_string_lossy());
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
        Ok(self.doc_count())    
    }

    fn add_document(&mut self, doc: &Document) -> Result<(),()> {
        let term_ids = self.analyzer.analyze(doc.get_content());
        let id = self.index.build_from(&term_ids);
        self.doc_meta.insert(id, doc.get_path().to_owned());
        Ok(())
    }

    pub fn compute_tf_idf(&mut self) -> Result<(),()> {
        if self.doc_meta.len() == 0 {
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

    pub fn stats(&self) -> Stats {
        Stats{
            index: self.index.stats(self.analyzer.get_dictionary()),
            analyzer: self.analyzer.stats(),
        }
    }

    pub fn exec_query(&self, 
        phrase_str: &str,
        ranking: RankingAlgorithm,
        ) -> Vec<&String>{
        
        let ignore_non_exist_term: bool;
        match ranking {
            RankingAlgorithm::ExactMatch => ignore_non_exist_term = false,
            _ => ignore_non_exist_term = true,
        }
        let term_ids = Query::parse(phrase_str, ignore_non_exist_term, &self.analyzer);
        let mut docs = vec![];
        let doc_scores = self.index.query(&term_ids, ranking);
        for doc in doc_scores {
            if let Some(doc_path) = self.doc_meta.get(&doc.docid){
                docs.push(doc_path);
            }
        }    
        docs
    }

}

#[test]
fn test_build_index() {
    let mut engine = Engine::new();
    let res = engine.build_index(&Path::new("./sample_corpus/romeo_juliet"));
    assert_eq!(res, Ok(5));
}

#[test]
fn test_save_and_load_index() {
    let mut engine = Engine::new();
    let res = engine.build_index_from(&Path::new("./sample_corpus/romeo_juliet"));
    assert_eq!(res, Ok(5));
    assert_eq!(engine.doc_count(), 5);
    let index_path = &Path::new(".rir/romeo_juliet1.idx");
    let _ = engine.save_to(index_path);
    let loaded_engine = Engine::load_from(index_path);
    assert_eq!(loaded_engine.doc_count(), 5);
    let docs = loaded_engine.exec_query("Quarrel sir", RankingAlgorithm::ExactMatch);
    use std::collections::HashSet;
    let doc_set: HashSet<&String> = HashSet::from_iter(docs);
    assert_eq!(doc_set, HashSet::from([&"./sample_corpus/romeo_juliet/a/1.txt".to_string(), &"./sample_corpus/romeo_juliet/a/2.txt".to_string()]));
}

#[test]
fn test_search_phrase() {
    let mut engine = Engine::new();
    let res = engine.build_index(&Path::new("./sample_corpus/romeo_juliet"));
    assert_eq!(res, Ok(5));
    let mut docs = engine.exec_query("Quarrel sir", RankingAlgorithm::ExactMatch);
    use std::collections::HashSet;
    let doc_set: HashSet<&String> = HashSet::from_iter(docs);
    assert_eq!(doc_set, HashSet::from([&"./sample_corpus/romeo_juliet/a/1.txt".to_string(), &"./sample_corpus/romeo_juliet/a/2.txt".to_string()]));
    docs = engine.exec_query("sir", RankingAlgorithm::ExactMatch);
    assert_eq!(docs.len(), 4);

    docs = engine.exec_query("non-exist", RankingAlgorithm::ExactMatch);
    assert_eq!(docs.len(), 0);

    docs = engine.exec_query("Sir non-exist", RankingAlgorithm::ExactMatch);
    assert_eq!(docs.len(), 0);

    docs = engine.exec_query("Sir", RankingAlgorithm::ExactMatch);
    assert_eq!(docs.len(), 4);

}

#[test]
fn test_vector_space_model() {
    let mut engine = Engine::new();
    let res = engine.build_index_from(&Path::new("./sample_corpus/romeo_juliet"));
    assert_eq!(res, Ok(5));
    assert_eq!(engine.doc_count(), 5);
    let index_path = &Path::new(".rir/romeo_juliet2.idx");
    let _ = engine.save_to(index_path);
    let loaded_engine = Engine::load_from(index_path);
    assert_eq!(loaded_engine.doc_count(), 5);
    let docs = loaded_engine.exec_query("Quarrel sir", RankingAlgorithm::VectorSpaceModel);
    use std::collections::HashSet;
    let doc_set: HashSet<&String> = HashSet::from_iter(docs);
    assert_eq!(doc_set, HashSet::from([
            &"./sample_corpus/romeo_juliet/a/2.txt".to_string(), 
            &"./sample_corpus/romeo_juliet/a/1.txt".to_string(),
            &"./sample_corpus/romeo_juliet/5.txt".to_string(), 
            &"./sample_corpus/romeo_juliet/b/3.txt".to_string(),
            ]));
}

#[test]
fn test_rank_bm25() {
    let mut engine = Engine::new();
    let res = engine.build_index_from(&Path::new("./sample_corpus/romeo_juliet"));
    assert_eq!(res, Ok(5));
    assert_eq!(engine.doc_count(), 5);
    let index_path = &Path::new(".rir/romeo_juliet3.idx");
    let _ = engine.save_to(index_path);
    let loaded_engine = Engine::load_from(index_path);
    assert_eq!(loaded_engine.doc_count(), 5);
    let docs = loaded_engine.exec_query("Quarrel sir", RankingAlgorithm::OkapiBM25);
    use std::collections::HashSet;
    let doc_set: HashSet<&String> = HashSet::from_iter(docs);
    assert_eq!(doc_set, HashSet::from([
            &"./sample_corpus/romeo_juliet/a/2.txt".to_string(), 
            &"./sample_corpus/romeo_juliet/a/1.txt".to_string(),
            &"./sample_corpus/romeo_juliet/5.txt".to_string(), 
            &"./sample_corpus/romeo_juliet/b/3.txt".to_string(),
            ]));
}

#[test]
fn test_rank_default() {
    let mut engine = Engine::new();
    let res = engine.build_index_from(&Path::new("./sample_corpus/romeo_juliet"));
    assert_eq!(res, Ok(5));
    assert_eq!(engine.doc_count(), 5);
    let index_path = &Path::new(".rir/romeo_juliet4.idx");
    let _ = engine.save_to(index_path);
    let loaded_engine = Engine::load_from(index_path);
    assert_eq!(loaded_engine.doc_count(), 5);
    let docs = loaded_engine.exec_query("Quarrel sir", RankingAlgorithm::Default);
    use std::collections::HashSet;
    let doc_set: HashSet<&String> = HashSet::from_iter(docs);
    assert_eq!(doc_set, HashSet::from([
            &"./sample_corpus/romeo_juliet/a/2.txt".to_string(), 
            &"./sample_corpus/romeo_juliet/a/1.txt".to_string(),
            &"./sample_corpus/romeo_juliet/5.txt".to_string(), 
            &"./sample_corpus/romeo_juliet/b/3.txt".to_string(),
            ]));

}