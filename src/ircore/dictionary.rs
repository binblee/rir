use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use super::index::TermId;

#[derive(Debug, Serialize, Deserialize)]
pub struct Dictionary {
    term_ids: HashMap<String, TermId>,
    next_id: TermId,
}

impl Dictionary {
    pub fn new() -> Self {
        Dictionary {
            term_ids: HashMap::new(),
            next_id: 1,
        }
    }
    pub fn add(&mut self, word: &str) -> TermId {
        let term_id = self.term_ids.entry(word.to_owned()).or_insert(self.next_id);
        if self.next_id == *term_id {
            self.next_id += 1;
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

    pub fn get_term_count(&self) -> usize {
        self.term_ids.len()
    }
}
