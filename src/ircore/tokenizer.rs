use unicode_segmentation::UnicodeSegmentation;
use serde::{Serialize, Deserialize};
use jieba_rs::Jieba;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum Language {
    English,
    Chinese,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Segmentator {
    lang: Language,
    #[serde(skip)]
    zh_seg: Jieba,
}

impl Segmentator {
    pub fn new() -> Self {
        Segmentator{
            lang: Language::English,
            zh_seg: Jieba::new(),
        }
    }

    pub fn set_language(&mut self, lang: Language){
        self.lang = lang;
    }

    pub fn get_language(&self) -> Language {
        return self.lang;
    }
    pub fn parse_tokens<'a>(&self, text: &'a str) -> Vec<&'a str>{
        match self.lang {
            Language::English => return text.unicode_words().collect(),
            Language::Chinese => {
                let raw_word_list = self.zh_seg.cut(text, false);
                let mut words = vec![];
                for raw_word in raw_word_list {
                    let mut chars = raw_word.chars();
                    match chars.next() {
                        Some(c) => {
                            if c.is_alphabetic() {
                                words.push(raw_word);
                            }
                        }
                        _ => (),
                    }
                    
                }
                return words;
            }
        }
        
    }
    
    pub fn normalize(&self, text: &str) -> String {
        match self.lang {
            Language::English => {
                return text.to_lowercase();
            },
            Language::Chinese => {
                return text.to_string(); // do nothing at this moment, 
            },
        }
    }    

}


#[test]
fn test_parse_tokens() {
    let text = "Quarrel sir! no, sir!";
    let latinseg = Segmentator::new();
    let normalized = latinseg.normalize(text);
    let tokens = latinseg.parse_tokens(&normalized);
    assert_eq!(tokens, vec!["quarrel", "sir", "no", "sir"]);
}

#[test]
fn test_parse_chinese() {
    let text = "滚滚长江东逝水，浪花淘尽英雄。";
    let mut seg = Segmentator::new();
    seg.set_language(Language::Chinese);
    let normalized = seg.normalize(text);
    let tokens = seg.parse_tokens(&normalized);
    // assert_eq!(tokens, vec!["滚滚", "长江", "东", "逝水", "，", "浪花", "淘", "尽", "英雄", "。"]);
    assert_eq!(tokens, vec!["滚滚", "长江", "东", "逝水", "浪花", "淘", "尽", "英雄"]);
}