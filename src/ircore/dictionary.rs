use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use super::index::TermId;

#[derive(Debug, Serialize, Deserialize)]
pub struct Dictionary {
    term_ids: HashMap<String, TermId>,
    terms: HashMap<TermId, String>,
    next_id: TermId,
}

pub struct DictionarySummary {
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
    pub fn get(&self, term:&str) -> Option<TermId> {
        if let Some(term_id) = self.term_ids.get(term) {
            Some(*term_id)
        }else{
            None
        }
    }

    pub fn get_ids(&self, terms: &Vec<&str>) -> (Vec<TermId>, Vec<String>){
        let mut term_ids = vec![];
        let mut unknown_terms = vec![];
        for term in terms {
            if let Some(id) = self.get(term){
                term_ids.push(id);
            }else{
                unknown_terms.push(String::from(*term));
            }
        }
        (term_ids, unknown_terms)
    }

    pub fn generate_ids(&mut self, terms: &Vec<&str>) -> Vec<TermId>{
        let mut term_ids = vec![];
        for term in terms {
            if let Some(id) = self.get(term){
                term_ids.push(id);
            }else{
                term_ids.push(self.add(term));
            }
        }
        term_ids
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

    pub fn summary(&self) -> DictionarySummary {
        DictionarySummary{
            term_count: self.term_ids.len() as u32,
        }
    }
}

#[test]
fn test_dictionary() {
    let mut dict = Dictionary::new();
    let mut id = dict.add("one");
    assert_eq!(id, 1);
    id = dict.add("two");
    assert_eq!(id, 2);
    id = dict.add("three");
    assert_eq!(id, 3);
    assert_eq!(dict.get("one"), Some(1));
    assert_eq!(dict.get("two"), Some(2));
    assert_eq!(dict.get("three"), Some(3));
    assert_eq!(dict.get("do-not-exist"), None);
    assert_eq!(dict.get_ids(&vec!["one", "three", "two"]), (vec![1,3,2], vec![]));
    assert_eq!(dict.get_ids(&vec!["one", "three", "two", "four"]), (vec![1,3,2], vec!["four".to_string()]));
    assert_eq!(dict.get_term_by_id(1), "one");
    assert_eq!(dict.get_term_by_id(2), "two");
    assert_eq!(dict.get_term_by_id(3), "three");
    assert_eq!(dict.get_term_by_id(4), "");
    assert_eq!(dict.get_term_count(), 3);
    assert_eq!(dict.generate_ids(&vec!["alpha","beta"]),vec![4,5]);
    assert_eq!(dict.get_ids(&vec!["one", "three", "two"]), (vec![1,3,2], vec![]));
    assert_eq!(dict.get_ids(&vec!["one", "three", "two", "four"]), (vec![1,3,2], vec!["four".to_string()]));
    assert_eq!(dict.get_term_by_id(1), "one");
    assert_eq!(dict.get_term_by_id(2), "two");
    assert_eq!(dict.get_term_by_id(3), "three");
    assert_eq!(dict.get_term_by_id(4), "alpha");
    assert_eq!(dict.get("one"), Some(1));
    assert_eq!(dict.get("two"), Some(2));
    assert_eq!(dict.get("three"), Some(3));
    assert_eq!(dict.get("alpha"), Some(4));
    assert_eq!(dict.get("beta"), Some(5));
    assert_eq!(dict.get("do-not-exist"), None);
    assert_eq!(dict.get_term_count(), 5);

}