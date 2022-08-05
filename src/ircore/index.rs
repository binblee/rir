
use std::collections::{HashMap, HashSet};

type TermId = u32;
pub type DocId = u32;
type TermOffset = u32;

#[derive(Debug)]
pub struct Posting {
    doc_id: DocId,
    term_frequency: u32,
    positions: Vec<TermOffset>,
}

#[derive(Debug)]
struct WordDic {
    term_ids: HashMap<String, TermId>,
    next_id: TermId,
}

impl WordDic {
    fn new() -> Self {
        WordDic {
            term_ids: HashMap::new(),
            next_id: 1,
        }
    }
    fn get_id_or_insert(&mut self, word: &str) -> TermId {
        let term_id = self.term_ids.entry(word.to_owned()).or_insert(self.next_id);
        if self.next_id == *term_id {
            self.next_id += 1;
        }
        *term_id
    }
    fn get_id(&self, word:&str) -> Option<TermId> {
        if let Some(word_id) = self.term_ids.get(word) {
            Some(*word_id)
        }else{
            None
        }
    }
}

#[derive(Debug)]
pub struct PositionList {
    // term string -> int(id)
    word_dict: WordDic,
    // document list,  termid -> positions
    postings_lists: HashMap<TermId, Vec<Posting>>,
    next_doc_id: DocId,
    // the number of documents in the collection containing the term (id)
    document_frequency: HashMap<TermId, u32>,
    // the number of times term(termid) appears in document(doc_id)
    term_frequency: HashMap<(TermId, DocId), u32>,
    // number of tokens of a document measured in tokens
    document_length: HashMap<DocId, u32>,
    total_document_length: u32,
    // average document length
    average_document_length: f32,
    // total number of documents
    document_count: u32,
}

pub trait SchemaDependIndex {
    fn new() -> Self;
    fn build_from(&mut self, tokens: Vec<&str>) -> DocId;
    // get docs
    fn docs(&self, term_id: TermId) -> Option<HashSet<DocId>>; 
    fn docs_contain_all(&self, term_list: &Vec<TermId>) -> Option<HashSet<DocId>>;
    // get term positions (for phase search)
    fn first(&self, doc:DocId, term:TermId) -> Option<TermOffset>;
    fn next(&self, doc:DocId, term:TermId, after_position:TermOffset) -> Option<TermOffset>;
    fn last(&self, doc:DocId, term:TermId) -> Option<TermOffset>;
    fn prev(&self, doc:DocId, term:TermId, before_position:TermOffset) -> Option<TermOffset>;
    fn next_phrase(&self, doc:DocId, phrase: &Vec<TermId>, position:TermOffset) -> Option<(TermOffset, TermOffset)>;
    fn all_phrase(&self, doc: DocId, phrase: &Vec<TermId>) -> Vec<(TermOffset, TermOffset)>;
    fn search_phase(&self, phase: Vec<&str>) -> Vec<DocId>;
    // helper functions
    fn binary_search(
        positions: &Vec<TermOffset> , low:usize, high: usize, current: u32,
        test_fn: fn(u32, u32) -> bool, retval_fn: fn(usize, usize) -> usize) -> usize;

}

impl SchemaDependIndex for PositionList {
    fn new() -> Self {
        PositionList{
            word_dict: WordDic::new(),
            postings_lists: HashMap::new(),
            next_doc_id: 1,
            document_frequency: HashMap::new(),
            term_frequency: HashMap::new(),
            document_length: HashMap::new(),
            total_document_length: 0,
            average_document_length: 0.0,
            document_count: 0
        }
    }

    fn build_from(&mut self, tokens: Vec<&str>) -> DocId {
        let doc_id = self.next_doc_id;
        self.next_doc_id += 1;
        let mut cached_term_id: HashSet<TermId> = HashSet::new();
        // update document length
        let document_length = tokens.len() as u32;
        self.document_length.entry(doc_id).or_insert(document_length);
        self.total_document_length += document_length;
        // update document count
        self.document_count += 1;
        // update average document length
        self.average_document_length = self.total_document_length as f32 / self.document_count as f32;
        // build position index
        for (seq, word) in tokens.into_iter().enumerate() {
            let term_offset = seq as TermOffset + 1;
            let term_id = self.word_dict.get_id_or_insert(word);
            let postings = self.postings_lists.entry(term_id).or_insert_with(Vec::new);
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
            self.term_frequency.entry((term_id, doc_id))
                .and_modify(|count| *count += 1)
                .or_insert(1);
            // update document frequency
            if !cached_term_id.contains(&term_id) {
                cached_term_id.insert(term_id);
                self.document_frequency.entry(term_id)
                .and_modify(|count| *count +=1)
                .or_insert(1);
            }
        }
        doc_id
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

    fn first(&self, doc:DocId, term:TermId) -> Option<TermOffset> {
        if self.document_frequency.contains_key(&term) {
            let posts = &self.postings_lists.get(&term).unwrap();
            for post in *posts {
                if post.doc_id == doc {
                    assert!(post.positions.len()>0);
                    return Some(post.positions[0]);
                }
            }
        }
        None
    }


    fn next(&self, doc:DocId, term:TermId, after_position:TermOffset) -> Option<TermOffset> {
        if self.document_frequency.contains_key(&term) {
            let posts = &self.postings_lists.get(&term).unwrap();
            for post in *posts {
                if post.doc_id == doc {
                    let positions = &post.positions;
                    assert!(positions.len()>0);
                    if *positions.last()? <= after_position {
                        return None;
                    }else if positions[0] > after_position {
                        return Some(positions[0]);
                    }else{
                        let target = Self::binary_search(
                            positions, 0, positions.len()-1, after_position,
                            |v1, v2 | v1 <= v2, 
                            |_, v2 | v2);
                        return Some(positions[target]);
                    }
                }
            }
        }
        None
    }

    fn last(&self, doc:DocId, term:TermId) -> Option<TermOffset> {
        if self.document_frequency.contains_key(&term) {
            let posts = &self.postings_lists.get(&term).unwrap();
            for post in *posts {
                if post.doc_id == doc {
                    assert!(post.positions.len()>0);
                    return Some(*post.positions.last()?);
                }
            }
        }
        None
    }
    fn prev(&self, doc:DocId, term:TermId, before_position:TermOffset) -> Option<TermOffset> {
        if self.document_frequency.contains_key(&term) {
            let posts = &self.postings_lists.get(&term).unwrap();
            for post in *posts {
                if post.doc_id == doc {
                    let positions = &post.positions;
                    assert!(positions.len()>0);
                    if positions[0] >= before_position {
                        return None;
                    }else if *positions.last()? < before_position {
                        return Some(*positions.last()?)
                    }else{
                        let target = Self::binary_search(
                            positions, 0, positions.len(), before_position,
                            |v1, v2 | v1 < v2, |v1, _ | v1
                        );
                        return Some(positions[target]);        
                    }
                }
            }
        }
        None
    }

    fn binary_search(
        positions: &Vec<TermOffset> , low:usize, high: usize, current: u32,
        test_fn: fn(u32, u32) -> bool, retval_fn: fn(usize, usize) -> usize) -> usize
    {
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

    fn next_phrase(
        &self, doc:DocId, phrase: &Vec<TermId>, position:TermOffset) 
        -> Option<(TermOffset, TermOffset)>{
            if phrase.len() <= 0 {
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
                        eprintln!("Error in reverse itration in nextPhase."); 
                        std::process::exit(1)
                    }
                }
            }
            if start < end && end - start == (phrase.len() - 1) as u32 {
                return Some((start, end));
            }else{
                return self.next_phrase(doc, phrase, start);
            }
    }

    fn all_phrase(&self, doc: DocId, phrase: &Vec<TermId>) -> Vec<(TermOffset, TermOffset)> {
        let mut result = Vec::new();
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

    fn search_phase(&self, phase: Vec<&str>) -> Vec<DocId> {
        let mut phase_ids = Vec::new();
        for word in phase {
            if let Some(id) = self.word_dict.get_id(word){
                phase_ids.push(id);
            }else{
                println!("unknown ignore: {}", word);
            }
        }
        let mut docs = Vec::new();
        if let Some(doc_set) = self.docs_contain_all(&phase_ids){
            docs = doc_set.into_iter().collect();
        }
        docs.sort(); // ranking for now...
        docs

    }

}

#[test]
fn test_index_from_string(){
    let mut idx = PositionList::new();
    let mut doc_id = idx.build_from(vec!["hello", "world", "hello", "世", "界", "你", "好", "你", "好"]);
    /*
    PositionList { 
        word_dict: WordDic { term_ids: {"hello": 1, "世": 3, "好": 6, "界": 4, "world": 2, "你": 5}, next_id: 7 }, 
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
    assert_eq!(idx.word_dict.term_ids.len(), 6);
    assert_eq!(idx.postings_lists.len(), idx.word_dict.term_ids.len());
    assert_eq!(idx.document_frequency.len(), idx.word_dict.term_ids.len());
    assert_eq!(idx.term_frequency.len(), 6);
    assert_eq!(idx.total_document_length, 9);
    assert_eq!(idx.average_document_length, 9.0);
    assert_eq!(idx.document_count,1);

    doc_id = idx.build_from(vec!["你", "好", "明", "天"]);
    /*
    PositionList { 
        word_dict: WordDic { term_ids: {"天": 8, "你": 5, "明": 7, "hello": 1, "世": 3, "好": 6, "界": 4, "world": 2}, next_id: 9 }, 
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
    assert_eq!(idx.word_dict.term_ids.len(), 8);
    assert_eq!(idx.postings_lists.len(), idx.word_dict.term_ids.len());
    assert_eq!(idx.document_frequency.len(), idx.word_dict.term_ids.len());
    assert_eq!(idx.term_frequency.len(), 10);
    assert_eq!(idx.total_document_length, 13);
    assert_eq!(idx.average_document_length, 6.5);
    assert_eq!(idx.document_count,2);
}

#[test]
fn test_docs_contain_term() {
    let mut idx = PositionList::new();
    let mut doc_id = idx.build_from(vec!["hello", "world", "hello", "世", "界", "你", "好", "你", "好"]);
    assert_eq!(doc_id, 1);
    doc_id = idx.build_from(vec!["你", "好", "明", "天"]);
    assert_eq!(doc_id, 2);
    let term_ids = vec![1,6];
    let mut doc_set = idx.docs_contain_all(&term_ids);
    assert_eq!(term_ids, vec![1,6]); //should be untouched
    assert_eq!(Some(HashSet::from([1])), doc_set);
    doc_set = idx.docs(6);
    assert_eq!(Some(HashSet::from([1,2])), doc_set);

}

#[test]
fn test_phase() {
    let mut idx = PositionList::new();
    let mut doc_id = idx.build_from(vec!["hello", "world", "hello", "世", "界", "你", "好", "你", "好"]);
    assert_eq!(doc_id, 1);
    doc_id = idx.build_from(vec!["你", "好", "明", "天"]);
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
    let mut phase_start_end = idx.all_phrase(1, &vec![5,6]);
    assert_eq!(phase_start_end, vec![(6, 7), (8, 9)]);
    phase_start_end = idx.all_phrase(2, &vec![5,6]);
    assert_eq!(phase_start_end, vec![(1, 2)]);
}

#[test]
fn test_search_phase() {
    let mut idx = PositionList::new();
    let mut doc_id = idx.build_from(vec!["hello", "world", "hello", "世", "界", "你", "好", "你", "好"]);
    assert_eq!(doc_id, 1);
    doc_id = idx.build_from(vec!["你", "好", "明", "天"]);
    assert_eq!(doc_id, 2);

    let phase_in_tokens = vec!["你", "好"];
    let docs = idx.search_phase(phase_in_tokens);
    assert_eq!(docs, vec![1,2]);

}