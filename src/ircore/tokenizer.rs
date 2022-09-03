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


#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(tokens, vec!["滚滚", "长江", "东", "逝水", "浪花", "淘", "尽", "英雄"]);
    }

    #[test]
    fn test_jieba() {
        use jieba_rs::Token;
        let jieba = jieba_rs::Jieba::new();
        let words = jieba.cut("人们发现，地球上海陆交界处的潮汐所具有的高度规律性正是由月亮的位置（和月相）控制的。", false);
        assert_eq!(words, vec!["人们", "发现", "，", "地球", "上", 
            "海陆", "交界处", "的", "潮汐", "所", "具有", "的", "高度", "规律性", 
            "正是", "由", "月亮", "的", "位置", "（", "和", "月相", "）", "控制", "的", "。"]);
        let tokens = jieba.tokenize("滚滚长江东逝水，浪花淘尽英雄。", jieba_rs::TokenizeMode::Search, true);
        assert_eq!(tokens, vec![
            Token { word: "滚滚", start: 0, end: 2 }, 
            Token { word: "长江", start: 2, end: 4 }, 
            Token { word: "东", start: 4, end: 5 }, 
            Token { word: "逝水", start: 5, end: 7 }, 
            Token { word: "，", start: 7, end: 8 }, 
            Token { word: "浪花", start: 8, end: 10 }, 
            Token { word: "淘尽", start: 10, end: 12 }, 
            Token { word: "英雄", start: 12, end: 14 }, 
            Token { word: "。", start: 14, end: 15 }]
        );
    }
}
