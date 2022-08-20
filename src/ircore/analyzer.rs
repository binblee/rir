use super::dictionary::{Dictionary, DictionarySummary};
use super::tokenizer::{normalize, parse_tokens};
use super::index::TermId;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Analyzer {
    dict: Dictionary,
}

pub struct AnalyzerSummary {
    pub dict: DictionarySummary,
}

impl Analyzer {
    pub fn new() -> Self {
        Analyzer{
            dict: Dictionary::new(),
        }
    }

    pub fn get_dictionary(&self) -> &Dictionary{
        &self.dict
    }
    pub fn analyze(&mut self, text: &str) -> Vec<TermId> {
        let text_normalized = normalize(&text);
        let tokens = parse_tokens(&text_normalized);
        let term_ids = self.dict.generate_ids(&tokens);
        term_ids
    }

    pub fn parse(&self, text: &str) -> (Vec<TermId>, Vec<String>) {
        let text_normalized = normalize(&text);
        let tokens = parse_tokens(&text_normalized);
        self.dict.get_ids(&tokens)
    }

    pub fn summary(&self) -> AnalyzerSummary {
        AnalyzerSummary{
            dict: self.dict.summary(),
        }
    }

    pub fn get_term_by_id(&self, tid: TermId) -> String {
        self.dict.get_term_by_id(tid)
    }
}

