use crate::ircore::ranking::DocScore;
use crate::ircore::index::pl::{PositionList, SchemaDependIndex};
use crate::ircore::{DocId, TermId, TermOffset};

pub trait PhraseMatch {
    fn search_phrase(&self, term_ids: &Vec<TermId>) -> Vec<DocScore>;   
}

impl PhraseMatch for PositionList {
    fn search_phrase(&self, term_ids: &Vec<TermId>) -> Vec<DocScore> {
        let mut scores = vec![];
        if let Some(doc_set) = self.docs_contain_all(&term_ids){
            let docs_contain_all:Vec<DocId> = doc_set.into_iter().collect();
            for doc in docs_contain_all {
                let positions = self.all_phrase(doc, &term_ids);
                if positions.len() > 0 {
                    scores.push(DocScore{
                        docid:doc,
                        score: positions.len() as f32,
                });
                }
            }
        }
        scores.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap().reverse() );   
        scores
    }
}

trait PhraseMatchHelpers {
    fn first(&self, doc:DocId, term:TermId) -> Option<TermOffset>;
    fn next(&self, doc:DocId, term:TermId, after_position:TermOffset) -> Option<TermOffset>;
    fn last(&self, doc:DocId, term:TermId) -> Option<TermOffset>;
    fn prev(&self, doc:DocId, term:TermId, before_position:TermOffset) -> Option<TermOffset>;
    fn next_phrase(
        &self, doc:DocId, phrase: &Vec<TermId>, position:TermOffset) 
        -> Option<(TermOffset, TermOffset)>;
    fn all_phrase(&self, doc: DocId, phrase: &Vec<TermId>) -> Vec<(TermOffset, TermOffset)>;
    fn binary_search(
        positions: &Vec<TermOffset> , low:usize, high: usize, current: TermOffset,
        test_fn: fn(TermOffset, TermOffset) -> bool, retval_fn: fn(usize, usize) -> usize) -> usize;
}

impl PhraseMatchHelpers for PositionList {

    fn first(&self, doc:DocId, term:TermId) -> Option<TermOffset> {
        if let Some(_term_in_doc) = self.get_document_frequency(term) {
            if let Some(positions) = self.get_positions(term, doc) {
                assert!(positions.len() > 0);
                return Some(positions[0]);
            }
        }
        None
    }


    fn next(&self, doc:DocId, term:TermId, after_position:TermOffset) -> Option<TermOffset> {
        if let Some(_term_in_doc) = self.get_document_frequency(term) {
            if let Some(positions) = self.get_positions(term, doc) {
                assert!(positions.len()>0);
                if *positions.last()? <= after_position {
                    return None;
                }else if positions[0] > after_position {
                    return Some(positions[0]);
                }else{
                    let target = Self::binary_search(
                        &positions, 0, positions.len()-1, after_position,
                        |v1, v2 | v1 <= v2, 
                        |_, v2 | v2);
                    return Some(positions[target]);
                }
            }
        }
        None
    }

    fn last(&self, doc:DocId, term:TermId) -> Option<TermOffset> {
        if let Some(_term_in_doc) = self.get_document_frequency(term) {
            if let Some(positions) = self.get_positions(term, doc) {
                assert!(positions.len() > 0);
                return Some(*positions.last()?);
            }
        }
        None
    }
    fn prev(&self, doc:DocId, term:TermId, before_position:TermOffset) -> Option<TermOffset> {
        if let Some(_term_in_doc) = self.get_document_frequency(term) {
            if let Some(positions) = self.get_positions(term, doc){
                if positions[0] >= before_position {
                    return None;
                }else if *positions.last()? < before_position {
                    return Some(*positions.last()?)
                }else{
                    let target = Self::binary_search(
                        &positions, 0, positions.len(), before_position,
                        |v1, v2 | v1 < v2, |v1, _ | v1
                    );
                    return Some(positions[target]);        
                }
            }
        }
        None
    }

    fn next_phrase(
        &self, doc:DocId, phrase: &Vec<TermId>, position:TermOffset) 
        -> Option<(TermOffset, TermOffset)>{
            if phrase.len() <= 1 {
                return None;
            }
            let mut end = position;
            for term in phrase.iter(){
                match self.next(doc, *term, end){
                    Some(pos) => end = pos,
                    None => return None,
                }
            }
            let mut start = end;
            for term in phrase.iter().rev().skip(1){
                match self.prev(doc, *term, start){
                    Some(pos) => start = pos,
                    None => {
                        assert!(false); // should not reach to this line
                    }
                }
            }
            if start < end && end - start == (phrase.len() - 1) as TermOffset {
                return Some((start, end));
            }else{
                return self.next_phrase(doc, phrase, start);
            }
    }

    fn all_phrase(&self, doc: DocId, phrase: &Vec<TermId>) -> Vec<(TermOffset, TermOffset)> {
        let mut result = Vec::new();
        // one word phrase
        if phrase.len() == 0 {
            return result;
        }else if phrase.len() == 1 {
            //only one token, return all positions in this doc
            if let Some(positions) = self.get_positions(phrase[0], doc){
                    for pos in positions {
                        result.push((*pos, *pos));
                    }
            }else{
                return result;
            }
        }
        // phrase that have at least two words
        let mut pos = 0;
        loop{
            match self.next_phrase(doc, phrase, pos) {
                Some(r) => {
                    result.push(r);
                    pos = r.0;
                },
                None => break
            }    
        }
        result
    }

    fn binary_search(
        positions: &Vec<TermOffset> , low:usize, high: usize, current: TermOffset,
        test_fn: fn(TermOffset, TermOffset) -> bool, retval_fn: fn(usize, usize) -> usize) -> usize {

        let mut mid:usize;
        let mut low_index = low;
        let mut high_index = high;
        while high_index - low_index > 1 {
            mid = (high_index + low_index)/2;
            if test_fn(positions[mid], current) {
                low_index = mid;
            }else{
                high_index = mid;
            }
        }
        retval_fn(low_index, high_index)
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::ircore::index::pl::{PositionList, SchemaDependIndex};
    use crate::ircore::token::dictionary::Dictionary;
    use std::collections::HashSet;

    #[test]
    fn test_prhase() {
        let mut idx = PositionList::new();
        let mut dict = Dictionary::new();
        let mut term_ids = dict.generate_ids(&vec!["hello", "world", "hello", "世", "界", "你", "好", "你", "好"]);
        let mut doc_id = idx.add_document(&term_ids);
        assert_eq!(doc_id, 1);
        term_ids = dict.generate_ids(&vec!["你", "好", "明", "天"]);
        doc_id = idx.add_document(&term_ids);
        assert_eq!(doc_id, 2);
        let mut res=0;
        if let Some(pos) = idx.first(1, 1){
            res = pos;
            assert_eq!(pos, 1);
        }
        if let Some(pos) = idx.next(1, 1, res){
            res = pos;
            assert_eq!(pos, 3);
        }

        if let Some(pos) = idx.last(1, 1){
            res = pos;
            assert_eq!(pos, 3);
        }
        if let Some(pos) = idx.prev(1, 1, res){
            assert_eq!(pos, 1);
        }

        let docs = idx.docs_contain_all(&vec![5,6]);
        assert_eq!(docs, Some(HashSet::from([1,2])));
        let mut phrase_start_end = idx.all_phrase(1, &vec![5,6]);
        assert_eq!(phrase_start_end, vec![(6, 7), (8, 9)]);
        phrase_start_end = idx.all_phrase(2, &vec![5,6]);
        assert_eq!(phrase_start_end, vec![(1, 2)]);
    }

    #[test]
    fn test_search_phrase() {
        let mut idx = PositionList::new();
        let mut dict = Dictionary::new();
        let mut term_ids = dict.generate_ids(&vec!["hello", "world", "hello", "世", "界", "你", "好", "你", "好"]);
        let mut doc_id = idx.add_document(&term_ids);
        assert_eq!(doc_id, 1);
        term_ids = dict.generate_ids(&vec!["你", "好", "明", "天"]);
        doc_id = idx.add_document(&term_ids);
        assert_eq!(doc_id, 2);

        // let mut phrase_in_tokens = vec!["你", "好"];
        let mut term_ids = vec![5, 6]; // vec!["你", "好"];
        let mut docs = idx.search_phrase(&term_ids);
        assert_eq!(docs.len(), 2);
        assert_eq!(term_ids.len(),2);

        term_ids = vec![5]; // vec!["你"];
        docs = idx.search_phrase(&term_ids);
        assert_eq!(docs.len(), 2);
        assert_eq!(term_ids.len(),1);

    }
}

#[cfg(test)]
mod helper_tests {
    use crate::ircore::index::pl::PositionList;
    use super::PhraseMatchHelpers;

    #[test]
    fn test_binary_search() {
        let positions = vec![5, 20, 35, 50];
        let target = PositionList::binary_search(&positions, 0, positions.len() -1 ,
            19, 
            |v1, v2 | v1 <= v2, 
            |_, v2 | v2);
        assert_eq!(target, 1);

        let target = PositionList::binary_search(&positions, 0, positions.len() -1 ,
            19, 
            |v1, v2 | v1 < v2, 
            |v1, _ | v1);
        assert_eq!(target, 0);
    }
}
