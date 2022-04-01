use std::collections::HashMap;
pub struct InvertedIndex {
    dict: HashMap<String, Vec<u32>>,
}

// position of term is u32 type, start from u32::MIN+1, which is 1
// end at u32::MAX-1
impl InvertedIndex {
    const DOC_BEGIN:u32 = u32::MIN;
    const DOC_END:u32 = u32::MAX;
        pub fn new() -> Self {
        InvertedIndex{dict: HashMap::new()}
    }
    pub fn insert(&mut self, term: String, position: u32){
        assert!(position != Self::DOC_BEGIN);
        assert!(position != Self::DOC_END);
        let entry = self.dict.entry(term).or_insert_with(Vec::new);
        entry.push(position);
    }
    pub fn first(&self, t: &String) -> Option<u32>{
        if self.dict.contains_key(t){
            let entry = self.dict.get(t)?;
            let position = entry.first()?;
            return Some(*position);
        }
        None
    }
    pub fn last(&self, t: &String) -> Option<u32>{
        if self.dict.contains_key(t){
            let entry = self.dict.get(t)?;
            let position = entry.last()?;
            return Some(*position);
        }
        None
    }
    pub fn next(&self, t: &String, pos:u32) -> Option<u32>{
        fn binary_search(postings_list: &Vec<u32> , low:usize, high: usize, current: u32) -> usize {
            let mut mid:usize;
            let mut low_index = low;
            let mut high_index = high;
            while high_index - low_index > 1 {
                mid = (high_index + low_index)/2;
                if postings_list[mid] <= current {
                    low_index = mid;
                }else{
                    high_index = mid;
                }
            }
            high_index
        }
        if self.dict.contains_key(t) {
            let entry = self.dict.get(t)?;
            let &last_pos = entry.last()?;
            if last_pos <= pos {
                return None;
            }
            let list_len = entry.len();
            assert!(list_len > 0);
            if entry[0] > pos {
                return Some(entry[0]);
            }else{
                let target = binary_search(entry, 0, list_len, pos);
                return Some(entry[target]);
            }
        }
        None
    }
    pub fn prev(&self, t: &String, pos:u32) -> Option<u32>{
        fn binary_search(postings_list: &Vec<u32> , low:usize, high: usize, current: u32) -> usize {
            let mut mid:usize;
            let mut low_index = low;
            let mut high_index = high;
            while high_index - low_index > 1 {
                mid = (high_index + low_index)/2;
                if postings_list[mid] < current {
                    low_index = mid;
                }else{
                    high_index = mid;
                }
            }
            low_index
        }
        if self.dict.contains_key(t) {
            let entry = self.dict.get(t)?;
            let list_len = entry.len();
            assert!(list_len > 0);
            if entry[0] >= pos {
                return None;
            }
            let last_value = *entry.last()?;
            if last_value < pos {
                return Some(last_value);
            }else{
                let target = binary_search(entry, 0, list_len, pos);
                return Some(entry[target]);
            }
        }
        None
    }
    fn len(&self, t: &String) -> usize {
        if self.dict.contains_key(t){
            let entry = self.dict.get(t).unwrap();
            return entry.len();
        }
        0
    }
    fn next_phrase(&self, phrase: &Vec<String>, position:u32) -> Option<(u32, u32)>{
        if phrase.len() <= 0 {
            return None;
        }
        let mut end = position;
        for t in phrase.iter(){
            match self.next(t, end){
                Some(pos) => end = pos,
                None => return None,
            }
        }
        let mut start = end;
        for t in phrase.iter().rev().skip(1){
            match self.prev(t, start){
                Some(pos) => start = pos,
                None => {
                    eprintln!("Error in reverse itration in nextPhase."); 
                    std::process::exit(1)
                }
            }
        }
        if start < end && end - start == (phrase.len() - 1) as u32 {
            return Some((start, end));
        }else{
            return self.next_phrase(phrase, start);
        }
    }

    pub fn all_phrase(&self, phrase: &Vec<String>) -> Vec<(u32,u32)> {
        let mut result = Vec::new();
        let mut pos = Self::DOC_BEGIN;
        loop{
            match self.next_phrase(phrase, pos) {
                Some(r) => {
                    result.push(r);
                    pos = r.0;
                },
                None => {break;}
            }    
        }
        result
    }

}

#[test]
fn test_inverted_index() {
    let inverted_index = setup();
    test_next_phrase(&inverted_index);
    test_all_phrase(&inverted_index);
    fn test_next_phrase(index: &InvertedIndex){
        assert_eq!(index.next_phrase(&vec!["first".to_string(),"witch".to_string()], 0),Some((745406, 745407)));
    }
    pub fn test_all_phrase(index: &InvertedIndex){
        let result = index.all_phrase(&vec!["first".to_string(),"witch".to_string()]);
        assert_eq!(result.len(),3);
        assert!(result[0].0 == 745406 && result[0].1 == 745407);
        assert!(result[1].0 == 745466 && result[1].1 == 745467);
        assert!(result[2].0 == 745501 && result[2].1 == 745502);
        let result_spam = index.all_phrase(&vec!["spam".to_string(), "spam".to_string(), "spam".to_string()]);
        assert_eq!(result_spam.len(),4);
        assert!(result_spam[0].0 == 1 && result_spam[0].1 == 3);
        assert!(result_spam[1].0 == 2 && result_spam[1].1 == 4);
        assert!(result_spam[2].0 == 3 && result_spam[2].1 == 5);
        assert!(result_spam[3].0 == 4 && result_spam[3].1 == 6);
    }
    fn setup() -> InvertedIndex {
        let mut inverted_index = InvertedIndex::new();
        inverted_index.insert("first".to_string(), 2205);
        inverted_index.insert("first".to_string(), 2268);
        inverted_index.insert("first".to_string(), 745406);
        inverted_index.insert("first".to_string(), 745466);
        inverted_index.insert("first".to_string(), 745501);
        inverted_index.insert("first".to_string(), 1271487);
        assert_eq!(inverted_index.len(&"first".to_string()),6);
        assert_eq!(inverted_index.first(&"first".to_string()), Some(2205));
        assert_eq!(inverted_index.last(&"first".to_string()), Some(1271487));
        assert_eq!(inverted_index.next(&"first".to_string(), 0), Some(2205));
        assert_eq!(inverted_index.next(&"first".to_string(), 5000), Some(745406));
        assert_eq!(inverted_index.next(&"first".to_string(), 745407), Some(745466));
        assert_eq!(inverted_index.next(&"first".to_string(), InvertedIndex::DOC_END), None);
        assert_eq!(inverted_index.next(&"first".to_string(), InvertedIndex::DOC_BEGIN), 
            inverted_index.first(&"first".to_string()));
        assert!(inverted_index.next(&"first".to_string(), 745466) != Some(745466));
        assert!(inverted_index.prev(&"first".to_string(), 745466) != Some(745466));
        assert_eq!(inverted_index.next(&"first".to_string(), 2000000), None);
        assert_eq!(inverted_index.prev(&"first".to_string(), 1000), None); 
        assert_eq!(inverted_index.prev(&"first".to_string(), 5000), Some(2268));
        assert_eq!(inverted_index.prev(&"first".to_string(), 2000000), Some(1271487));
        assert_eq!(inverted_index.prev(&"first".to_string(), InvertedIndex::DOC_BEGIN), None);
        assert_eq!(inverted_index.prev(&"first".to_string(), InvertedIndex::DOC_END), 
            inverted_index.last(&"first".to_string()));
        assert_eq!(inverted_index.first(&"sth invalid".to_string()), None);
        assert_eq!(inverted_index.last(&"sth invalid".to_string()), None);
        assert_eq!(inverted_index.next(&"sth invalid".to_string(), 2000000), None);
        assert_eq!(inverted_index.prev(&"sth invalid".to_string(), 1000), None); 
        assert_eq!(inverted_index.len(&"sth invalid".to_string()),0);
        inverted_index.insert("hurlyburly".to_string(), 316669);
        inverted_index.insert("hurlyburly".to_string(), 745434);
        assert_eq!(inverted_index.len(&"hurlyburly".to_string()),2);
        assert_eq!(inverted_index.first(&"hurlyburly".to_string()), Some(316669));
        assert_eq!(inverted_index.last(&"hurlyburly".to_string()), Some(745434));
        inverted_index.insert("witch".to_string(), 1598);
        inverted_index.insert("witch".to_string(), 27555);
        inverted_index.insert("witch".to_string(), 745407);
        inverted_index.insert("witch".to_string(), 745429);
        inverted_index.insert("witch".to_string(), 745451);
        inverted_index.insert("witch".to_string(), 745467);
        inverted_index.insert("witch".to_string(), 745502);
        inverted_index.insert("witch".to_string(), 1245276);
        inverted_index.insert("spam".to_string(), 1);
        inverted_index.insert("spam".to_string(), 2);
        inverted_index.insert("spam".to_string(), 3);
        inverted_index.insert("spam".to_string(), 4);
        inverted_index.insert("spam".to_string(), 5);
        inverted_index.insert("spam".to_string(), 6);
        assert_eq!(inverted_index.first(&"witch".to_string()), Some(1598));
        assert_eq!(inverted_index.last(&"witch".to_string()), Some(1245276));
        assert_eq!(inverted_index.len(&"witch".to_string()), 8);
        inverted_index
    }
}