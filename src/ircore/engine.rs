use crate::ircore::index::pl::{SchemaDependIndex, PositionList, IndexStats};
use crate::ircore::{DocId, RankingAlgorithm};
use crate::ircore::token::analyzer::{Analyzer, AnalyzerStats};
use std::path::Path;
use std::collections::{HashMap};
use serde::{Serialize, Deserialize};
use crate::ircore::doc::Document;
use crate::ircore::query::Query;
use crate::ircore::ranking::Scorer;
use crate::ircore::doc::doc_parser::DocParser;
use crate::ircore::utils::serialize;
use std::io;

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
    const SERIALIZE_NAME_ANALYZER:&'static str = "idx.al";
    const SERIALIZE_NAME_DOCMETA: &'static str = "idx.dm";

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

    pub fn load_from(path: &str) -> Self {
        let mut engine = Self::new();
        engine.index = PositionList::load_from(path);
        engine.load_analyzer(path);
        engine.load_docmeta(path);
        return engine;
    }

    fn load_analyzer(&mut self, path_str: &str){
        let path = Path::new(path_str).join(Path::new(Self::SERIALIZE_NAME_ANALYZER));
        let mut encoded:Vec<u8> = vec![];
        if let Ok(reloaded_al) = serialize::read_file(&path, &mut encoded) {
            self.analyzer = reloaded_al;
        }else{
            self.analyzer = Analyzer::new();
        }
    }

    fn load_docmeta(&mut self, path_str: &str){
        let path = Path::new(path_str).join(Path::new(Self::SERIALIZE_NAME_DOCMETA));
        let mut encoded:Vec<u8> = vec![];
        if let Ok(reloaded_dm) = serialize::read_file(&path, &mut encoded) {
            self.doc_meta = reloaded_dm;
        }else{
            self.doc_meta = HashMap::new();
        }
    }

    pub fn build_index_from(&mut self, path: &str) -> Result<usize, ()> {
        for docs in DocParser::new(path).docs(){
            for doc in docs {
                self.add_document(&doc).unwrap();
                if self.index.get_document_count() % 1000 == 0 {
                    log::debug!("{}", self.index.get_document_count());
                }    
            }
        }
        log::debug!("build index completed, number of doc: {}", self.doc_count());
        return Ok(self.doc_count());
    }

    fn add_document(&mut self, doc: &Document) -> Result<(),()> {
        let term_ids = self.analyzer.analyze(doc.get_content());
        let id = self.index.add_document(&term_ids);
        self.doc_meta.insert(id, doc.get_path().to_owned());
        Ok(())
    }

    pub fn save_to(&mut self, path_str: &str) -> io::Result<()> {
        self.index.save_to(path_str)?;
        self.save_analyzer(path_str)?;
        self.save_docmeta(path_str)?;
        Ok(())
    }

    pub fn save_analyzer(&mut self, path_str: &str) -> io::Result<()> {
        let path = Path::new(path_str).join(Path::new(Self::SERIALIZE_NAME_ANALYZER));
        serialize::write_file(&path, &self.analyzer)?;
        log::debug!("analyzer save to {}", path.to_string_lossy());
        Ok(())
    }

    pub fn save_docmeta(&mut self, path_str: &str) -> io::Result<()> {
        let path = Path::new(path_str).join(Path::new(Self::SERIALIZE_NAME_DOCMETA));
        serialize::write_file(&path, &self.doc_meta)?;        
        log::debug!("docmeta save to {}", path.to_string_lossy());
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
        let doc_scores = self.index.score(&term_ids, ranking);
        for doc in doc_scores {
            if let Some(doc_path) = self.doc_meta.get(&doc.docid){
                docs.push(doc_path);
            }
        }    
        docs
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_index() {
        let mut engine = Engine::new();
        let res = engine.build_index_from("./sample_corpus/romeo_juliet");
        assert_eq!(res, Ok(5));
    }

    #[test]
    fn test_save_and_load_index() {
        let mut engine = Engine::new();
        let res = engine.build_index_from("./sample_corpus/romeo_juliet");
        assert_eq!(res, Ok(5));
        assert_eq!(engine.doc_count(), 5);
        let index_path = ".rir/romeo_juliet1.idx";
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
        let res = engine.build_index_from("./sample_corpus/romeo_juliet");
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
        let res = engine.build_index_from("./sample_corpus/romeo_juliet");
        assert_eq!(res, Ok(5));
        assert_eq!(engine.doc_count(), 5);
        let index_path = ".rir/romeo_juliet2.idx";
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
        let res = engine.build_index_from("./sample_corpus/romeo_juliet");
        assert_eq!(res, Ok(5));
        assert_eq!(engine.doc_count(), 5);
        let index_path = ".rir/romeo_juliet3.idx";
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
        let res = engine.build_index_from("./sample_corpus/romeo_juliet");
        assert_eq!(res, Ok(5));
        assert_eq!(engine.doc_count(), 5);
        let index_path = ".rir/romeo_juliet4.idx";
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

    #[test]
    fn test_chinese_text_index() {
        let mut engine = Engine::new();
        let res = engine.build_index_from("./sample_corpus/sanguo");
        assert_eq!(res, Ok(19));
        assert_eq!(engine.doc_count(), 19);
        let index_path = ".rir/sanguo.idx";
        let _ = engine.save_to(index_path);
        let loaded_engine = Engine::load_from(index_path);
        assert_eq!(loaded_engine.doc_count(), 19);
        let docs = loaded_engine.exec_query("刘备", RankingAlgorithm::ExactMatch);
        use std::collections::HashSet;
        let doc_set: HashSet<&String> = HashSet::from_iter(docs);
        assert_eq!(doc_set, HashSet::from([&"./sample_corpus/sanguo/9.txt".to_string()]));
        let docs = loaded_engine.exec_query("桃园结义", RankingAlgorithm::Default);
        let doc_set: HashSet<&String> = HashSet::from_iter(docs);
        assert_eq!(doc_set, HashSet::from([
            &"./sample_corpus/sanguo/1.txt".to_string(),
            &"./sample_corpus/sanguo/9.txt".to_string(),
            &"./sample_corpus/sanguo/8.txt".to_string(),
            ]));

    }

    #[test]
    fn test_build_index_from_json_files() {
        let mut engine = Engine::new();
        let res = engine.build_index_from("./sample_corpus/wiki_zh");
        assert_eq!(res, Ok(2));
        assert_eq!(engine.doc_count(), 2);
        let index_path = ".rir/wiki_zh.idx";
        let _ = engine.save_to(index_path);
        let loaded_engine = Engine::load_from(index_path);
        assert_eq!(loaded_engine.doc_count(), 2);
        let docs = loaded_engine.exec_query("数学", RankingAlgorithm::ExactMatch);
        use std::collections::HashSet;
        let doc_set: HashSet<&String> = HashSet::from_iter(docs);
        assert_eq!(doc_set, HashSet::from(
            [&"./sample_corpus/wiki_zh/wiki_1".to_string(),
            &"./sample_corpus/wiki_zh/wiki_2".to_string()]));
    }

}