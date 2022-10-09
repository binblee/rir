use std::collections::{HashMap, HashSet};
use std::cmp::Reverse;
use serde::{Serialize, Deserialize};
use crate::ircore::utils::sparse_vector::{SparseVector, SparseVectorOp};
use crate::ircore::token::dictionary::Dictionary;
use crate::ircore::{DocId, TermId, TermOffset};

type Positions = Vec<TermOffset>;
#[derive(Debug, Serialize, Deserialize)]
pub struct Posting {
    doc_id: DocId,
    term_frequency: u32,
    positions: Positions,
}

impl Posting {
    pub fn get_doc_id(&self) -> DocId {
        self.doc_id
    }
    pub fn get_positions(&self) -> &Positions {
        &self.positions
    }
}


type PositingList = HashMap<TermId, Vec<Posting>>;


#[derive(Debug, Serialize, Deserialize)]
pub struct PositionList {
    // document list,  termid -> positions
    postings_lists: PositingList,
    next_doc_id: DocId,
    // the number of documents in the collection containing the term (id)
    document_frequency: HashMap<TermId, u32>,
    // the number of times term(termid) appears in document(doc_id)
    term_frequency: HashMap<(TermId, DocId), u32>,
    // number of tokens of a document measured in tokens
    // value doc_id - 1 is used as vector index
    document_length: Vec<u32>,
    total_document_length: u64,
    // average document length
    average_document_length: f32,
    // total number of documents
    document_count: usize,
    // doc-term list, for TF-IDF computing
    #[serde(skip)]
    doc_terms: HashMap<DocId, HashSet<TermId>>,
}

pub struct IndexStats {
    // total document length in tokens
    pub total_document_length: u64,
    // average document length
    pub average_document_length: f32,
    // total number of documents
    pub document_count: usize,
    pub term_freq: Vec<(TermId, String, u32)>,
}

pub trait SchemaDependIndex {
    fn new() -> Self;
    fn add_document(&mut self, term_ids: &Vec<TermId>) -> DocId;
    fn next_doc_id(&mut self) -> DocId;
    // getters
    // get: positition list for one term in doc
    fn get_positions(&self, term: TermId, doc: DocId) -> Option<&Positions>;
    // get: number of term occurences in whole collection
    fn get_term_occurences_num(&self, term: TermId) -> u32;
    // get: total number of document
    fn get_document_count(&self) -> usize;
    // get: average document length
    fn get_average_document_length(&self) -> f32;
    // get: document length
    fn get_document_length(&self, doc: DocId) -> u32;
    // get: term frequency in specified document
    fn get_term_frequency(&self, term: TermId, doc: DocId) -> Option<&u32>;
    // get: the number of documents in the collection containing the term (id)
    fn get_document_frequency(&self, term: TermId) -> Option<&u32>;

    // docs contain the term
    fn docs(&self, term_id: TermId) -> Option<HashSet<DocId>>; 
    // docs contain all terms
    fn docs_contain_all(&self, term_list: &Vec<TermId>) -> Option<HashSet<DocId>>;
    // docs contain any of the terms
    fn docs_contain_any(&self, term_list: &Vec<TermId>) -> HashSet<DocId>;
    fn is_valid_doc_id(&self, doc_id: DocId) -> bool;
    // TF-IDF related
    fn get_doc_tfidf_vector(&self, doc: DocId) -> SparseVector;
    fn get_phrase_tfidf_vector(&self, phrase: &Vec<TermId>) -> Box<SparseVector>;
    // Statistics
    fn stats(&self, dict: &Dictionary) -> IndexStats;
    // Validate if index is good
    fn validate(&self) -> bool;
    // Rebuild index after load from index file
    fn rebuild(&mut self) -> bool;
}

impl SchemaDependIndex for PositionList {
    fn new() -> Self {
        PositionList{
            // dict: Dictionary::new(),
            postings_lists: HashMap::new(),
            next_doc_id: 0,
            document_frequency: HashMap::new(),
            term_frequency: HashMap::new(),
            document_length: vec![],
            total_document_length: 0,
            average_document_length: 0.0,
            document_count: 0,
            doc_terms: HashMap::new(),
        }
    }

    fn get_term_occurences_num(&self, term: TermId) -> u32{
        if let Some(term_postings_list) = self.postings_lists.get(&term){
            let occ_num = term_postings_list.into_iter().fold(0u32, |sum, posting| sum + posting.term_frequency);
            return occ_num;
        }
        0
    }

    fn get_positions(&self, term: TermId, doc: DocId) -> Option<&Positions> {
        let posts = self.postings_lists.get(&term).unwrap();
        for post in posts {
            if post.get_doc_id() == doc {
                let positions = post.get_positions();
                assert!(positions.len()>0);
                return Some(positions);
            }
        }
        None
    }


    // get: total number of document
    fn get_document_count(&self) -> usize {
        self.document_count
    }
    
    // get: average document length
    fn get_average_document_length(&self) -> f32 {
        self.average_document_length
    }

    // get: document length
    fn get_document_length(&self, doc: DocId) -> u32 {
        self.document_length[doc as usize - 1]
    }

    // get: term frequency in specified document
    fn get_term_frequency(&self, term: TermId, doc: DocId) -> Option<&u32> {
        self.term_frequency.get(&(term, doc))
    }

    // get: the number of documents in the collection containing the term (id)
    fn get_document_frequency(&self, term: TermId) -> Option<&u32> {
        self.document_frequency.get(&term)
    }



    fn next_doc_id(&mut self) -> DocId {
        self.next_doc_id += 1;
        self.next_doc_id
    }

    fn add_document(&mut self, term_ids: &Vec<TermId>) -> DocId {
        let doc_id = self.next_doc_id();
        let mut cached_term_id: HashSet<TermId> = HashSet::new();
        // update document length
        let document_length = term_ids.len() as u32;
        self.document_length.push(document_length);
        self.total_document_length += document_length as u64;
        // update document count
        self.document_count += 1;
        // update average document length
        self.average_document_length = self.total_document_length as f32 / self.document_count as f32;
        // build position index
        for (seq, tid) in term_ids.into_iter().enumerate() {
            let term_offset = seq as TermOffset + 1;
            let postings = self.postings_lists.entry(*tid).or_insert_with(Vec::new);
            if postings.len() == 0 || postings.last().unwrap().doc_id != doc_id {
                postings.push(Posting{
                    doc_id: doc_id,
                    term_frequency: 1,
                    positions: vec![term_offset],
                })
            }else{
                let post = postings.last_mut().unwrap();
                assert_eq!(post.doc_id, doc_id);
                post.term_frequency += 1;
                post.positions.push(term_offset);
            }
            // update term frequency
            self.term_frequency.entry((*tid, doc_id))
                .and_modify(|count| *count += 1)
                .or_insert(1);
            // update document frequency
            if !cached_term_id.contains(tid) {
                cached_term_id.insert(*tid);
                self.document_frequency.entry(*tid)
                .and_modify(|count| *count +=1)
                .or_insert(1);
            }
            // update doc-term map
            let doc_terms_entry = self.doc_terms.entry(doc_id)
                .or_insert_with(HashSet::new);
            doc_terms_entry.insert(*tid);
        }
        doc_id
    }

    // Summary
    fn stats(&self, dict: &Dictionary) -> IndexStats {
        let mut idx_info = IndexStats {
            total_document_length: self.total_document_length,
            average_document_length: self.average_document_length,
            document_count: self.document_count,
            term_freq: vec![],
        };
        let mut term_freq_map:HashMap<TermId, u32> = HashMap::new();
        for ((tid, _), &freq) in &self.term_frequency{
            term_freq_map.entry(*tid)
                .and_modify(| count | *count += freq)
                .or_insert(freq);
        }
        for (tid, freq) in &term_freq_map {
            idx_info.term_freq.push((*tid, dict.get_term_by_id(*tid), *freq));
        }
        idx_info.term_freq.sort_by_key(|itm| Reverse(itm.2) );
        idx_info
    }
    
    fn docs(&self, term_id: TermId) -> Option<HashSet<DocId>> {
        let mut docid_set;
        if self.postings_lists.contains_key(&term_id) {
            let postings = self.postings_lists.get(&term_id).unwrap();
            docid_set = HashSet::new();
            for post in postings {
                docid_set.insert(post.doc_id);
            }
            Some(docid_set)
        }else{
            None
        }
    }

    fn docs_contain_all(&self, term_list: &Vec<TermId>) -> Option<HashSet<DocId>> {
        let mut doc_set = HashSet::new();
        let mut initialized = false;
        for term in term_list {
            if let Some(res_set) = self.docs(*term) {
                if initialized {
                    doc_set = &doc_set & &res_set;
                }else{
                    doc_set = res_set;
                    initialized = true;
                }
            }
        }
        if initialized {
            Some(doc_set)
        }else{
            None
        }
    }

    fn docs_contain_any(&self, term_list: &Vec<TermId>) -> HashSet<DocId> {
        let mut doc_set:HashSet<DocId> = HashSet::new();
        for term in term_list {
            if let Some(res_set) = self.docs(*term) {
                    doc_set = &doc_set | &res_set;
            }
        }
        doc_set
    }

    fn is_valid_doc_id(&self, doc_id: DocId) -> bool {
        doc_id >= 1 && doc_id <= self.document_count as DocId + 1 
    }

    // TF = log(ftd) + 1 if ftd > 0, 0 otherwise
    // IDF = log(N/Nt)
    //   ftd: inverted term document frequency (document_frequency[doc_id])
    //   N: total count of document (term_frequency[term_id, doc_id])
    //   Nt: total count of document that contain term t (document_count)
    fn get_doc_tfidf_vector(&self, doc: DocId) -> SparseVector {
        assert!(self.is_valid_doc_id(doc));
        let mut tfidf_vec = SparseVector::new();
        if let Some(term_set) = self.doc_terms.get(&doc) {
            for term in term_set {
                let freq = *self.term_frequency.get(&(*term, doc)).unwrap() as f32;
                let term_tfidf = (freq.log2() + 1f32 ) * 
                    (self.document_count as f32 / *self.document_frequency.get(term).unwrap() as f32).log2();
                tfidf_vec.vec_set(*term, term_tfidf);
            }
        }
        tfidf_vec.vec_normalize();
        return tfidf_vec;
    }

    // compute query string's TF-IDF vector
    fn get_phrase_tfidf_vector(&self, terms: &Vec<TermId>) -> Box<SparseVector> {
        let mut query_term_freq:HashMap<TermId, u32> = HashMap::new();
        for &tid in terms {
            query_term_freq.entry(tid)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
        let mut query_tfidf = SparseVector::new();
        for &tid in query_term_freq.keys() {
            let freq = *query_term_freq.get(&tid).unwrap() as f32;
            let term_tfidf = (freq.log2() + 1f32 ) * 
                (self.document_count as f32 / *self.document_frequency.get(&tid).unwrap() as f32).log2();
            query_tfidf.vec_set(tid, term_tfidf);
        }
        query_tfidf.vec_normalize();
        Box::new(query_tfidf)
    }

    // Validate if index is good
    fn validate(&self) -> bool {
        true
    }

    // Rebuild index after load from index file
    fn rebuild(&mut self) -> bool {
        // reload doc_terms
        // type PositingList = HashMap<TermId, Vec<Posting>>;
        // doc_terms: HashMap<DocId, HashSet<TermId>>
        if self.doc_terms.len() == 0 {
            for term_id in self.postings_lists.keys() {
                let postings = self.postings_lists.get(term_id).unwrap();
                for posting in postings {
                    let doc_terms_entry = 
                        self.doc_terms.entry(posting.doc_id)
                        .or_insert_with(HashSet::new);
                    doc_terms_entry.insert(*term_id);
                }
            }
        }
        true
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ircore::token::dictionary::Dictionary;

    #[test]
    fn test_index_from_string(){
        let mut idx = PositionList::new();
        let mut dict = Dictionary::new();
        let mut term_ids = dict.generate_ids(&vec!["hello", "world", "hello", "世", "界", "你", "好", "你", "好"]);
        let mut doc_id = idx.add_document(&term_ids);
        /*
        PositionList { 
            dict: Dictionary { term_ids: {"hello": 1, "世": 3, "好": 6, "界": 4, "world": 2, "你": 5}, next_id: 7 }, 
            postings_lists: {1: [Posting { doc_id: 1, term_frequency: 2, positions: [1, 3] }], 5: [Posting { doc_id: 1, term_frequency: 2, positions: [6, 8] }], 6: [Posting { doc_id: 1, term_frequency: 2, positions: [7, 9] }], 3: [Posting { doc_id: 1, term_frequency: 1, positions: [4] }], 2: [Posting { doc_id: 1, term_frequency: 1, positions: [2] }], 4: [Posting { doc_id: 1, term_frequency: 1, positions: [5] }]}, 
            next_doc_id: 2, 
            document_frequency: {1: 1, 6: 1, 4: 1, 2: 1, 3: 1, 5: 1}, 
            term_frequency: {(3, 1): 1, (1, 1): 2, (2, 1): 1, (4, 1): 1, (6, 1): 2, (5, 1): 2}, 
            document_length: {1: 9}, 
            total_document_length: 9, 
            average_document_length: 9.0, 
            document_count: 1 
        }
        */
        assert_eq!(doc_id, 1);
        assert_eq!(dict.get_term_count(), 6);
        assert_eq!(idx.postings_lists.len(), dict.get_term_count());
        assert_eq!(idx.document_frequency.len(), dict.get_term_count());
        assert_eq!(idx.term_frequency.len(), 6);
        assert_eq!(idx.total_document_length, 9);
        assert_eq!(idx.average_document_length, 9.0);
        assert_eq!(idx.document_count,1);
        if let Some(doc_terms_entry1) = idx.doc_terms.get(&(1 as DocId)){
            assert!(doc_terms_entry1.contains(&(1 as DocId)));
            assert!(doc_terms_entry1.contains(&(2 as DocId)));
            assert!(doc_terms_entry1.contains(&(3 as DocId)));
            assert!(doc_terms_entry1.contains(&(4 as DocId)));
            assert!(doc_terms_entry1.contains(&(5 as DocId)));
            assert!(doc_terms_entry1.contains(&(6 as DocId)));
        }else{
            assert!(false);
        }

        term_ids = dict.generate_ids(&vec!["你", "好", "明", "天"]);
        doc_id = idx.add_document(&term_ids);
        /*
        PositionList { 
            dict: Dictionary { term_ids: {"天": 8, "你": 5, "明": 7, "hello": 1, "世": 3, "好": 6, "界": 4, "world": 2}, next_id: 9 }, 
            postings_lists: {1: [Posting { doc_id: 1, term_frequency: 2, positions: [1, 3] }], 3: [Posting { doc_id: 1, term_frequency: 1, positions: [4] }], 4: [Posting { doc_id: 1, term_frequency: 1, positions: [5] }], 7: [Posting { doc_id: 2, term_frequency: 1, positions: [3] }], 5: [Posting { doc_id: 1, term_frequency: 2, positions: [6, 8] }, Posting { doc_id: 2, term_frequency: 1, positions: [1] }], 6: [Posting { doc_id: 1, term_frequency: 2, positions: [7, 9] }, Posting { doc_id: 2, term_frequency: 1, positions: [2] }], 8: [Posting { doc_id: 2, term_frequency: 1, positions: [4] }], 2: [Posting { doc_id: 1, term_frequency: 1, positions: [2] }]}, 
            next_doc_id: 3, 
            document_frequency: {6: 2, 4: 1, 3: 1, 1: 1, 8: 1, 5: 2, 2: 1, 7: 1}, 
            term_frequency: {(8, 2): 1, (3, 1): 1, (4, 1): 1, (7, 2): 1, (5, 2): 1, (1, 1): 2, (2, 1): 1, (6, 1): 2, (6, 2): 1, (5, 1): 2}, 
            document_length: {2: 4, 1: 9}, 
            total_document_length: 13, 
            average_document_length: 6.5, 
            document_count: 2 }    
        */
        assert_eq!(doc_id, 2);    
        assert_eq!(dict.get_term_count(), 8);
        assert_eq!(idx.postings_lists.len(), dict.get_term_count());
        assert_eq!(idx.document_frequency.len(), dict.get_term_count());
        assert_eq!(idx.term_frequency.len(), 10);
        assert_eq!(idx.total_document_length, 13);
        assert_eq!(idx.average_document_length, 6.5);
        assert_eq!(idx.document_count,2);
        if let Some(doc_terms_entry1) = idx.doc_terms.get(&(1 as DocId)){
            assert!(doc_terms_entry1.contains(&(1 as DocId)));
            assert!(doc_terms_entry1.contains(&(2 as DocId)));
            assert!(doc_terms_entry1.contains(&(3 as DocId)));
            assert!(doc_terms_entry1.contains(&(4 as DocId)));
            assert!(doc_terms_entry1.contains(&(5 as DocId)));
            assert!(doc_terms_entry1.contains(&(6 as DocId)));
            assert!(!doc_terms_entry1.contains(&(7 as DocId)));
            assert!(!doc_terms_entry1.contains(&(8 as DocId)));
        }else{
            assert!(false);
        }
        if let Some(doc_terms_entry2) = idx.doc_terms.get(&(2 as DocId)){
            assert!(doc_terms_entry2.contains(&(5 as DocId)));
            assert!(doc_terms_entry2.contains(&(6 as DocId)));
            assert!(doc_terms_entry2.contains(&(7 as DocId)));
            assert!(doc_terms_entry2.contains(&(8 as DocId)));
        }else{
            assert!(false);
        }


        // test getter functions
        assert_eq!(idx.get_term_occurences_num(2), 1); //world
        assert_eq!(idx.get_term_occurences_num(1), 2); //hello
        assert_eq!(idx.get_term_occurences_num(6), 3); //好
        assert_eq!(idx.get_term_occurences_num(7), 1); //明
    }

    #[test]
    fn test_docs_contain_term() {
        let mut idx = PositionList::new();
        let mut dict = Dictionary::new();
        let mut term_ids = dict.generate_ids(&vec!["hello", "world", "hello", "世", "界", "你", "好", "你", "好"]);
        let mut doc_id = idx.add_document(&term_ids);
        assert_eq!(doc_id, 1);
        term_ids = dict.generate_ids(&vec!["你", "好", "明", "天"]);
        doc_id = idx.add_document(&term_ids);
        assert_eq!(doc_id, 2);
        // contain all
        let mut term_ids = vec![1,6];
        let mut doc_set = idx.docs_contain_all(&term_ids);
        assert_eq!(term_ids, vec![1,6]); //should be untouched
        assert_eq!(Some(HashSet::from([1])), doc_set);
        // contain one term
        doc_set = idx.docs(6);
        assert_eq!(Some(HashSet::from([1,2])), doc_set);
        // contain any
        term_ids = vec![7];
        let doc_set = idx.docs_contain_any(&term_ids);
        assert_eq!(doc_set, HashSet::from([2]));
        term_ids = vec![7, 5];
        let doc_set = idx.docs_contain_any(&term_ids);
        assert_eq!(doc_set, HashSet::from([1, 2]));
        // invalid term id
        term_ids = vec![100];
        let doc_set = idx.docs_contain_any(&term_ids);
        assert_eq!(doc_set.len(), 0);
        term_ids = vec![7,100,7];
        let doc_set = idx.docs_contain_any(&term_ids);
        assert_eq!(doc_set, HashSet::from([2]));
    }

    #[test]
    fn test_reload_index() {
        let mut idx = PositionList::new();
        let mut dict = Dictionary::new();
        let mut term_ids = dict.generate_ids(&vec!["hello", "world", "hello", "世", "界", "你", "好", "你", "好"]);
        let mut doc_id = idx.add_document(&term_ids);
        assert_eq!(doc_id, 1);
        /*
        PositionList { 
            dict: Dictionary { term_ids: {"hello": 1, "世": 3, "好": 6, "界": 4, "world": 2, "你": 5}, next_id: 7 }, 
            postings_lists: {1: [Posting { doc_id: 1, term_frequency: 2, positions: [1, 3] }], 5: [Posting { doc_id: 1, term_frequency: 2, positions: [6, 8] }], 6: [Posting { doc_id: 1, term_frequency: 2, positions: [7, 9] }], 3: [Posting { doc_id: 1, term_frequency: 1, positions: [4] }], 2: [Posting { doc_id: 1, term_frequency: 1, positions: [2] }], 4: [Posting { doc_id: 1, term_frequency: 1, positions: [5] }]}, 
            next_doc_id: 2, 
            document_frequency: {1: 1, 6: 1, 4: 1, 2: 1, 3: 1, 5: 1}, 
            term_frequency: {(3, 1): 1, (1, 1): 2, (2, 1): 1, (4, 1): 1, (6, 1): 2, (5, 1): 2}, 
            document_length: {1: 9}, 
            total_document_length: 9, 
            average_document_length: 9.0, 
            document_count: 1 
        }
        */
        term_ids = dict.generate_ids(&vec!["你", "好", "明", "天"]);
        doc_id = idx.add_document(&term_ids);
        /*
        PositionList { 
            dict: Dictionary { term_ids: {"天": 8, "你": 5, "明": 7, "hello": 1, "世": 3, "好": 6, "界": 4, "world": 2}, next_id: 9 }, 
            postings_lists: {1: [Posting { doc_id: 1, term_frequency: 2, positions: [1, 3] }], 3: [Posting { doc_id: 1, term_frequency: 1, positions: [4] }], 4: [Posting { doc_id: 1, term_frequency: 1, positions: [5] }], 7: [Posting { doc_id: 2, term_frequency: 1, positions: [3] }], 5: [Posting { doc_id: 1, term_frequency: 2, positions: [6, 8] }, Posting { doc_id: 2, term_frequency: 1, positions: [1] }], 6: [Posting { doc_id: 1, term_frequency: 2, positions: [7, 9] }, Posting { doc_id: 2, term_frequency: 1, positions: [2] }], 8: [Posting { doc_id: 2, term_frequency: 1, positions: [4] }], 2: [Posting { doc_id: 1, term_frequency: 1, positions: [2] }]}, 
            next_doc_id: 3, 
            document_frequency: {6: 2, 4: 1, 3: 1, 1: 1, 8: 1, 5: 2, 2: 1, 7: 1}, 
            term_frequency: {(8, 2): 1, (3, 1): 1, (4, 1): 1, (7, 2): 1, (5, 2): 1, (1, 1): 2, (2, 1): 1, (6, 1): 2, (6, 2): 1, (5, 1): 2}, 
            document_length: {2: 4, 1: 9}, 
            total_document_length: 13, 
            average_document_length: 6.5, 
            document_count: 2 }    
        */
        assert_eq!(doc_id, 2);    
        assert_eq!(dict.get_term_count(), 8);
        // clear doc_terms to trigger rebuild
        idx.doc_terms = HashMap::new();
        let rebuild_res = idx.rebuild();
        assert!(rebuild_res);
        assert_eq!(idx.postings_lists.len(), dict.get_term_count());
        assert_eq!(idx.document_frequency.len(), dict.get_term_count());
        assert_eq!(idx.term_frequency.len(), 10);
        assert_eq!(idx.total_document_length, 13);
        assert_eq!(idx.average_document_length, 6.5);
        assert_eq!(idx.document_count,2);
        if let Some(doc_terms_entry1) = idx.doc_terms.get(&(1 as DocId)){
            assert!(doc_terms_entry1.contains(&(1 as DocId)));
            assert!(doc_terms_entry1.contains(&(2 as DocId)));
            assert!(doc_terms_entry1.contains(&(3 as DocId)));
            assert!(doc_terms_entry1.contains(&(4 as DocId)));
            assert!(doc_terms_entry1.contains(&(5 as DocId)));
            assert!(doc_terms_entry1.contains(&(6 as DocId)));
            assert!(!doc_terms_entry1.contains(&(7 as DocId)));
            assert!(!doc_terms_entry1.contains(&(8 as DocId)));
        }else{
            assert!(false);
        }
        if let Some(doc_terms_entry2) = idx.doc_terms.get(&(2 as DocId)){
            assert!(doc_terms_entry2.contains(&(5 as DocId)));
            assert!(doc_terms_entry2.contains(&(6 as DocId)));
            assert!(doc_terms_entry2.contains(&(7 as DocId)));
            assert!(doc_terms_entry2.contains(&(8 as DocId)));
        }else{
            assert!(false);
        }


        // test getter functions
        assert_eq!(idx.get_term_occurences_num(2), 1); //world
        assert_eq!(idx.get_term_occurences_num(1), 2); //hello
        assert_eq!(idx.get_term_occurences_num(6), 3); //好
        assert_eq!(idx.get_term_occurences_num(7), 1); //明
    }
}