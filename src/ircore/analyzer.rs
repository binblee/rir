use crate::ircore::dictionary::{Dictionary, DictionaryStats};
use crate::ircore::tokenizer::{Segmentator, Language};
use crate::ircore::common::TermId;
use serde::{Serialize, Deserialize};
use whatlang::{Detector, Lang};

#[derive(Debug, Serialize, Deserialize)]
pub struct Analyzer {
    dict: Dictionary,
    seg: Segmentator,
    lang_detected: bool,
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
            lang_detected: false,
        }
    }

    pub fn detect_language(&mut self, doc_content: &str){
        if !self.lang_detected {
            let allowlist = vec![Lang::Eng, Lang::Cmn];
            let detector = Detector::with_allowlist(allowlist);
            let lang = detector.detect_lang(doc_content);
            match lang {
                Some(Lang::Cmn) => self.set_language(Language::Chinese),
                _ => (), // default English
            }
            self.lang_detected = true;
        }
    }

    pub fn set_language(&mut self, lang: Language){
        self.seg.set_language(lang)
    }

    pub fn get_language(&self) -> Language{
        self.seg.get_language()
    }

    pub fn get_dictionary(&self) -> &Dictionary{
        &self.dict
    }
    pub fn analyze(&mut self, text: &str) -> Vec<TermId> {
        self.detect_language(text);
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
        match self.get_language() {
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
    fn test_analyze_english() {
        let mut analyzer = Analyzer::new();
        let term_ids = analyzer.analyze("Do you quarrel, sir?");
        assert_eq!(analyzer.get_language(), Language::English);
        assert_eq!(term_ids, vec![1, 2, 3, 4]);
    }


    #[test]
    fn test_analyze_chinese() {
        let mut analyzer = Analyzer::new();
        let term_ids = analyzer.analyze("滚滚长江东逝水，浪花淘尽英雄。");
        assert_eq!(analyzer.get_language(), Language::Chinese);
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

    #[test]
    fn test_whatlang() {
        let allowlist = vec![Lang::Eng, Lang::Cmn];
        let detector = Detector::with_allowlist(allowlist);
        let mut lang = detector.detect_lang("There is no reason not to learn Esperanto.");
        assert_eq!(lang, Some(Lang::Eng));
        lang = detector.detect_lang("宴桃园豪杰三结义　斩黄巾英雄首立功");
        assert_eq!(lang, Some(Lang::Cmn));
    }

}