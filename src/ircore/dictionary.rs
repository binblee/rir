use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use super::index::TermId;

#[derive(Debug, Serialize, Deserialize)]
pub struct Dictionary {
    term_ids: HashMap<String, TermId>,
    terms: HashMap<TermId, String>,
    next_id: TermId,
}

pub struct DictionaryInfo {
    pub term_count: u32,
}

impl Dictionary {
    pub fn new() -> Self {
        Dictionary {
            term_ids: HashMap::new(),
            terms: HashMap::new(),
            next_id: 1,
        }
    }
    pub fn add(&mut self, word: &str) -> TermId {
        let term_id = self.term_ids.entry(word.to_owned()).or_insert(self.next_id);
        if self.next_id == *term_id {
            self.next_id += 1;
        }
        if !self.terms.contains_key(term_id){
            self.terms.insert(*term_id, word.to_owned());
        }
        *term_id
    }
    pub fn get(&self, word:&str) -> Option<TermId> {
        if let Some(word_id) = self.term_ids.get(word) {
            Some(*word_id)
        }else{
            None
        }
    }

    pub fn get_term_by_id(&self, tid: TermId) -> String {
        if let Some(term_str) = self.terms.get(&tid) {
            return term_str.clone();
        }else{
            return "".to_string();
        }
    }

    pub fn get_term_count(&self) -> usize {
        self.term_ids.len()
    }

    pub fn info(&self) -> DictionaryInfo {
        DictionaryInfo{
            term_count: self.term_ids.len() as u32,
        }
    }
}
