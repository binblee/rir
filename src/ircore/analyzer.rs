use super::dictionary::{Dictionary, DictionaryStats};
use super::tokenizer::{Segmentator, Language};
use super::common::TermId;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Analyzer {
    dict: Dictionary,
    seg: Segmentator,
}

pub struct AnalyzerStats {
    pub dict: DictionaryStats,
    pub lang: String,
}

impl Analyzer {
    pub fn new() -> Self {
        Analyzer{
            dict: Dictionary::new(),
            seg: Segmentator::new(),
        }
    }

    pub fn set_language(&mut self, lang: Language){
        self.seg.set_language(lang);
    }

    pub fn get_dictionary(&self) -> &Dictionary{
        &self.dict
    }
    pub fn analyze(&mut self, text: &str) -> Vec<TermId> {
        let text_normalized = self.seg.normalize(&text);
        let tokens = self.seg.parse_tokens(&text_normalized);
        let term_ids = self.dict.generate_ids(&tokens);
        term_ids
    }

    pub fn parse(&self, text: &str) -> (Vec<TermId>, Vec<String>) {
        let text_normalized = self.seg.normalize(&text);
        let tokens = self.seg.parse_tokens(&text_normalized);
        self.dict.get_ids(&tokens)
    }

    pub fn stats(&self) -> AnalyzerStats {
        let lang_str;
        match self.seg.get_language() {
            Language::English => lang_str = String::from("English"),
            Language::Chinese => lang_str = String::from("Chinese"),
        }
        AnalyzerStats{
            dict: self.dict.stats(),
            lang: lang_str,
        }
    }

    pub fn get_term_by_id(&self, tid: TermId) -> String {
        self.dict.get_term_by_id(tid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_analyzer() {
        let mut analyzer = Analyzer::new();
        let mut term_ids = analyzer.analyze("Do you quarrel, sir?");
        assert_eq!(analyzer.seg.get_language(), Language::English);
        assert_eq!(term_ids, vec![1, 2, 3, 4]);
        term_ids = analyzer.analyze("Quarrel sir! no, sir!");
        assert_eq!(term_ids, vec![3, 4, 5, 4]);

        let (term_known, unknown_terms) = analyzer.parse("quarrel sir");
        assert_eq!(term_known, vec![3, 4]);
        assert_eq!(unknown_terms, Vec::<String>::from(vec![]));        

        let (term_known, unknown_terms) = analyzer.parse("quarrel sir Cool");
        assert_eq!(term_known, vec![3, 4]);
        assert_eq!(unknown_terms, vec!["cool"]);        
    }

    #[test]
    fn test_analyze_chinese() {
        let mut analyzer = Analyzer::new();
        analyzer.set_language(Language::Chinese);
        assert_eq!(analyzer.seg.get_language(), Language::Chinese);
        let term_ids = analyzer.analyze("滚滚长江东逝水，浪花淘尽英雄。");
        assert_eq!(term_ids, vec![1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn test_unicode_functions() {
        assert!(' '.is_whitespace());
        assert!('a'.is_alphabetic());
        assert!('中'.is_alphabetic());
        assert!('a'.is_ascii_alphabetic());
        assert!(!'中'.is_ascii_alphabetic());
        assert!(!'，'.is_alphabetic());
        
    }

}