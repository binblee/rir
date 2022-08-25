use std::collections::{HashMap, HashSet};
use std::cmp::Reverse;
use serde::{Serialize, Deserialize};
use super::utils::sparse_vector::{SparseVector, SparseVectorOp};
use super::dictionary::Dictionary;
use super::common::{DocId, TermId, TermOffset, RankingAlgorithm};


#[derive(Debug, Serialize, Deserialize)]
pub struct Posting {
    doc_id: DocId,
    term_frequency: u32,
    positions: Vec<TermOffset>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct PositionList {
    // document list,  termid -> positions
    postings_lists: HashMap<TermId, Vec<Posting>>,
    next_doc_id: DocId,
    // the number of documents in the collection containing the term (id)
    document_frequency: HashMap<TermId, u32>,
    // the number of times term(termid) appears in document(doc_id)
    term_frequency: HashMap<(TermId, DocId), u32>,
    // number of tokens of a document measured in tokens
    // TODO: use vector
    document_length: HashMap<DocId, u32>,
    total_document_length: u32,
    // average document length
    average_document_length: f32,
    // total number of documents
    document_count: u32,
    // TF-IDF for all documents
    tf_idf_matrix: Vec<SparseVector>,
}

pub struct IndexStats {
    // total document length in tokens
    pub total_document_length: u32,
    // average document length
    pub average_document_length: f32,
    // total number of documents
    pub document_count: u32,
    pub term_freq: Vec<(TermId, String, u32)>,
}

pub trait SchemaDependIndex {
    fn new() -> Self;
    fn build_from(&mut self, term_ids: &Vec<TermId>) -> DocId;
    fn next_doc_id(&mut self) -> DocId;
    // get: number of term occurences in whole collection
    fn get_term_occurences_num(&self, term: TermId) -> u32;
    // docs contain the term
    fn docs(&self, term_id: TermId) -> Option<HashSet<DocId>>; 
    // docs contain all terms
    fn docs_contain_all(&self, term_list: &Vec<TermId>) -> Option<HashSet<DocId>>;
    // docs contain any of the terms
    fn docs_contain_any(&self, term_list: &Vec<TermId>) -> HashSet<DocId>;
    fn is_valid_doc_id(&self, doc_id: DocId) -> bool;
    // get term positions (for phrase search)
    fn first(&self, doc:DocId, term:TermId) -> Option<TermOffset>;
    fn next(&self, doc:DocId, term:TermId, after_position:TermOffset) -> Option<TermOffset>;
    fn last(&self, doc:DocId, term:TermId) -> Option<TermOffset>;
    fn prev(&self, doc:DocId, term:TermId, before_position:TermOffset) -> Option<TermOffset>;
    fn next_phrase(&self, doc:DocId, phrase: &Vec<TermId>, position:TermOffset) -> Option<(TermOffset, TermOffset)>;
    fn all_phrase(&self, doc: DocId, phrase: &Vec<TermId>) -> Vec<(TermOffset, TermOffset)>;
    fn search_phrase(&self, phrase: &Vec<TermId>) -> Vec<DocScore>;
    // TF-IDF compute
    fn compute_tf_idf(&mut self) -> Result<(),()>;
    fn rank_cosine(&self, terms: &Vec<TermId>) -> Vec<DocScore>;
    // BM25
    fn rank_bm25(&self, terms: &Vec<TermId>) -> Vec<DocScore>;
    // LMD
    fn rank_lmd(&self, terms: &Vec<TermId>) -> Vec<DocScore>;
    // Statistics
    fn stats(&self, dict: &Dictionary) -> IndexStats;
    // query wrapper
    fn query(&self, terms: &Vec<TermId>, ranking: RankingAlgorithm) -> Vec<DocScore>;
    // helper functions
    fn binary_search(
        positions: &Vec<TermOffset> , low:usize, high: usize, current: u32,
        test_fn: fn(u32, u32) -> bool, retval_fn: fn(usize, usize) -> usize) -> usize;

}

pub struct DocScore {
    pub docid: DocId,
    pub score: f32,
}

impl SchemaDependIndex for PositionList {
    fn new() -> Self {
        PositionList{
            // dict: Dictionary::new(),
            postings_lists: HashMap::new(),
            next_doc_id: 0,
            document_frequency: HashMap::new(),
            term_frequency: HashMap::new(),
            document_length: HashMap::new(),
            total_document_length: 0,
            average_document_length: 0.0,
            document_count: 0,
            tf_idf_matrix: vec![SparseVector::new()],
        }
    }

    fn get_term_occurences_num(&self, term: TermId) -> u32{
        if let Some(term_postings_list) = self.postings_lists.get(&term){
            let occ_num = term_postings_list.into_iter().fold(0u32, |sum, posting| sum + posting.term_frequency);
            return occ_num;
        }
        0
    }


    fn next_doc_id(&mut self) -> DocId {
        self.next_doc_id += 1;
        self.next_doc_id
    }

    fn build_from(&mut self, term_ids: &Vec<TermId>) -> DocId {
        //TODO to be replaced by function call
        let doc_id = self.next_doc_id();
        let mut cached_term_id: HashSet<TermId> = HashSet::new();
        // update document length
        let document_length = term_ids.len() as u32;
        self.document_length.entry(doc_id).or_insert(document_length);
        self.total_document_length += document_length;
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
        doc_id >= 1 && doc_id <= self.document_count + 1 
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
                        eprintln!("Error in reverse itration in next Phrase."); 
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
        // one word phrase
        if phrase.len() == 0 {
            return result;
        }else if phrase.len() == 1 {
            //only one token, return all positions in this doc
            if self.postings_lists.contains_key(&phrase[0]) {
                let positings = self.postings_lists.get(&phrase[0]).unwrap();
                for post in positings {
                    //TODO binary search
                    if post.doc_id == doc {
                        for pos in &post.positions {
                            result.push((*pos, *pos));
                        }
                        break;
                    }
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

    fn compute_tf_idf(&mut self) -> Result<(),()>{
        assert_eq!(self.document_count, self.document_length.len() as u32);
        for _ in 0..self.document_count {
            let tf_idf_vector = SparseVector::new();
            self.tf_idf_matrix.push(tf_idf_vector);
        }
        let doc_count = self.document_count as f32;
        for (term_id, doc_id) in self.term_frequency.keys() {
            assert!(*doc_id <= self.document_count + 1);
            let term_freq = *self.term_frequency.get( &(*term_id, *doc_id) ).unwrap() as f32;
            assert!(self.document_frequency.contains_key(term_id));
            let doc_freq = *self.document_frequency.get(term_id).unwrap() as f32;
            let tfidf_first_pass = (term_freq.log2() + 1.0) * (doc_count / doc_freq).log2();
            self.tf_idf_matrix[*doc_id as usize].vec_set(*term_id, tfidf_first_pass);
        }
        for i in 1..self.document_count + 1 {
            self.tf_idf_matrix[i as usize].vec_normalize();
        }
        Ok(())
    }
    fn rank_cosine(&self, term_ids: &Vec<TermId>) -> Vec<DocScore> {
        let mut scores = vec![];
        if term_ids.len() == 0 {
            return scores;
        }
        // compute query string's TF-IDF vector
        let mut query_term_freq:HashMap<TermId, u32> = HashMap::new();
        for &tid in term_ids {
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
        // go through all documents that contains at least one term
        let docs_contain_any = self.docs_contain_any(&term_ids);
        for doc_id in docs_contain_any {
            if self.is_valid_doc_id(doc_id){
                let doc_tfidf = &self.tf_idf_matrix[doc_id as usize];
                let vec_distance = query_tfidf.vec_dot(doc_tfidf);
                scores.push(DocScore{docid: doc_id, score: vec_distance});
            }
        }
        // sort by socres
        scores.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap().reverse() );
        scores
    }

    // The BM25 algorithm
    // for all term t sum qt * ftd*(k1+1)/(k1*(1-b+b*(ld/lvag)) + ftd) * log(N/Nt)
    //   qt: query term frequency
    //   ftd: inverted term document frequency, document_length[doc_id]
    //   k1: weight saturation factor, default 1.2
    //   b: level of normalization of document length, default 0.75
    //   N: total count of document
    //   Nt: total count of document that contain term t
    fn rank_bm25(&self, term_ids: &Vec<TermId>) -> Vec<DocScore> {
        let mut scores = vec![];
        if term_ids.len() == 0 {
            return scores;
        }
        // find out query term frequency
        let mut query_term_freq:HashMap<TermId, u32> = HashMap::new();
        for &tid in term_ids {
            query_term_freq.entry(tid)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
        // compute scores
        let k1 = 1.2f32;
        let k1plus1 = k1 + 1.0;
        let b = 0.75f32;
        let document_count = self.document_count as f32;
        let lavg = self.average_document_length as f32;
        let docs_contain_any = self.docs_contain_any(&term_ids);
        for docid in docs_contain_any {
            let ld = *self.document_length.get(&docid).unwrap() as f32;
            let k1_b_ld_lavg = k1*(1.0-b+b*(ld/lavg));
            let mut score = 0f32;
            for &tid in query_term_freq.keys() {
                let qt = *query_term_freq.get(&tid).unwrap() as f32;
                if let Some(ftd_ref) = self.term_frequency.get(&(tid, docid)){
                    let nt = *self.document_frequency.get(&tid).unwrap() as f32;
                    let idf = (document_count/nt).log2(); 
                    let ftd = *ftd_ref as f32;
                    score += qt * ftd * k1plus1 / (k1_b_ld_lavg + ftd) * idf;    
                }
            }
            scores.push(DocScore{docid: docid, score:score});  
        }
        // sort by socres
        scores.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap().reverse() );
        scores
    }

    // LMD - language modeling with Dirichlet smoothing
    // for all term t: sum(qt * log(1 + ftd * N / lt)) - n * log(1 + ld / lavg)
    //   qt: query term frequency
    //   ftd: inverted term document frequency, document_length[doc_id]
    //   N: total count of document
    //   lt: number of times term t occurs in the collection
    //   n: equals to sum(all qt), is the number of tokens in the query
    //   ld: length of the document d, measured in tokens
    //   lavg: average length of all documents in the collection
    fn rank_lmd(&self, terms: &Vec<TermId>) -> Vec<DocScore> {
        let mut scores = vec![];
        if terms.len() == 0 {
            return scores;
        }
        // find out query term frequency
        let mut query_term_freq:HashMap<TermId, u32> = HashMap::new();
        for &tid in terms {
            query_term_freq.entry(tid)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
        let document_count = self.document_count as f32; // N
        let lavg = self.average_document_length as f32;
        let query_token_num = terms.len() as f32; // n
        let docs_contain_any = self.docs_contain_any(&terms);
        for docid in docs_contain_any {
            let ld = *self.document_length.get(&docid).unwrap() as f32;
            let mut score = 0f32;
            for &tid in query_term_freq.keys() {
                let qt = *query_term_freq.get(&tid).unwrap() as f32;
                if let Some(ftd_ref) = self.term_frequency.get(&(tid, docid)){
                    let ftd = *ftd_ref as f32; 
                    let lt = self.get_term_occurences_num(tid) as f32;
                    score += (1f32 + ftd * document_count / lt).log2() * qt;
                }
            }
            score -= (1f32 + ld / lavg).log2() * query_token_num;
            scores.push(DocScore{docid: docid, score:score});  
        }
        // sort by socres
        scores.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap().reverse() );
        scores
    }


    fn query(&self, terms: &Vec<TermId>, ranking: RankingAlgorithm) -> Vec<DocScore> {
        let docs = vec![];
        if terms.len() == 0 {
            return docs;
        }
        let scorer: fn(&PositionList, &Vec<TermId>) -> Vec<DocScore>;
        match ranking {
            RankingAlgorithm::Default => scorer = PositionList::rank_bm25,
            RankingAlgorithm::ExactMatch => scorer = PositionList::search_phrase,
            RankingAlgorithm::VectorSpaceModel => scorer = PositionList::rank_cosine,
            RankingAlgorithm::OkapiBM25 => scorer = PositionList::rank_bm25,  
            RankingAlgorithm::LMD => scorer = PositionList::rank_lmd,       
        }
        let doc_scores = scorer(&self, &terms);
        doc_scores
    }



}

#[test]
fn test_index_from_string(){
    let mut idx = PositionList::new();
    use super::dictionary::{Dictionary};
    let mut dict = Dictionary::new();
    let mut term_ids = dict.generate_ids(&vec!["hello", "world", "hello", "世", "界", "你", "好", "你", "好"]);
    let mut doc_id = idx.build_from(&term_ids);
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

    term_ids = dict.generate_ids(&vec!["你", "好", "明", "天"]);
    doc_id = idx.build_from(&term_ids);
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

    // test getter functions
    assert_eq!(idx.get_term_occurences_num(2), 1); //world
    assert_eq!(idx.get_term_occurences_num(1), 2); //hello
    assert_eq!(idx.get_term_occurences_num(6), 3); //好
    assert_eq!(idx.get_term_occurences_num(7), 1); //明
}

#[test]
fn test_docs_contain_term() {
    let mut idx = PositionList::new();
    use super::dictionary::{Dictionary};
    let mut dict = Dictionary::new();
    let mut term_ids = dict.generate_ids(&vec!["hello", "world", "hello", "世", "界", "你", "好", "你", "好"]);
    let mut doc_id = idx.build_from(&term_ids);
    assert_eq!(doc_id, 1);
    term_ids = dict.generate_ids(&vec!["你", "好", "明", "天"]);
    doc_id = idx.build_from(&term_ids);
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
fn test_prhase() {
    let mut idx = PositionList::new();
    use super::dictionary::{Dictionary};
    let mut dict = Dictionary::new();
    let mut term_ids = dict.generate_ids(&vec!["hello", "world", "hello", "世", "界", "你", "好", "你", "好"]);
    let mut doc_id = idx.build_from(&term_ids);
    assert_eq!(doc_id, 1);
    term_ids = dict.generate_ids(&vec!["你", "好", "明", "天"]);
    doc_id = idx.build_from(&term_ids);
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
    use super::dictionary::{Dictionary};
    let mut dict = Dictionary::new();
    let mut term_ids = dict.generate_ids(&vec!["hello", "world", "hello", "世", "界", "你", "好", "你", "好"]);
    let mut doc_id = idx.build_from(&term_ids);
    assert_eq!(doc_id, 1);
    term_ids = dict.generate_ids(&vec!["你", "好", "明", "天"]);
    doc_id = idx.build_from(&term_ids);
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

#[test]
fn test_vector_space_model(){
    let mut idx = PositionList::new();
    use super::dictionary::{Dictionary};
    let mut dict = Dictionary::new();
    let mut term_ids = dict.generate_ids(&vec!["do", "you", "quarrel", "sir"]);
    let mut doc_id = idx.build_from(&term_ids);
    assert_eq!(doc_id, 1);
    term_ids = dict.generate_ids(&vec!["quarrel", "sir", "no", "sir"]);
    doc_id = idx.build_from(&term_ids);
    assert_eq!(doc_id, 2);
    term_ids = dict.generate_ids(&vec!["if", "you", "do", "sir", "i", "am", "for", "you", 
    "i", "serve", "as", "good", "a", "man", "as", "you"]);
    doc_id = idx.build_from(&term_ids);
    assert_eq!(doc_id, 3);
    term_ids = dict.generate_ids(&vec!["no", "better"]);
    doc_id = idx.build_from(&term_ids);
    assert_eq!(doc_id, 4);
    term_ids = dict.generate_ids(&vec!["well", "sir"]);
    doc_id = idx.build_from(&term_ids);
    assert_eq!(doc_id, 5);
    assert!(idx.is_valid_doc_id(0) == false);
    let tfidf_ok = idx.compute_tf_idf();
    assert_eq!(idx.document_count, idx.document_length.len() as u32);
    assert_eq!(tfidf_ok, Ok(()));
    let term_ids = vec![3, 4]; //vec!["quarrel", "sir"];
    let docs = idx.rank_cosine(&term_ids);
    assert_eq!(term_ids.len(), 2);
    assert_eq!(docs.len(), 4);
    // DocumentID 1    2    3    4    5
    // Similarity 0.59 0.73 0.01 0.00 0.03
    let epsilon = 0.005;
    assert_eq!( docs[0].docid, 2 );
    assert!( (docs[0].score - 0.73).abs() <= epsilon );
    assert_eq!( docs[1].docid, 1 );
    assert!( (docs[1].score - 0.59).abs() <= epsilon );
    assert_eq!( docs[2].docid, 5 );
    assert!( (docs[2].score - 0.03).abs() <= epsilon );
    assert_eq!( docs[3].docid, 3 );
    assert!( (docs[3].score - 0.01).abs() <= epsilon );
}

#[test]
fn test_rank_bm25(){
    let mut idx = PositionList::new();
    use super::dictionary::{Dictionary};
    let mut dict = Dictionary::new();
    let mut term_ids = dict.generate_ids(&vec!["do", "you", "quarrel", "sir"]);
    let mut doc_id = idx.build_from(&term_ids);
    assert_eq!(doc_id, 1);
    term_ids = dict.generate_ids(&vec!["quarrel", "sir", "no", "sir"]);
    doc_id = idx.build_from(&term_ids);
    assert_eq!(doc_id, 2);
    term_ids = dict.generate_ids(&vec!["if", "you", "do", "sir", "i", "am", "for", "you", 
    "i", "serve", "as", "good", "a", "man", "as", "you"]);
    doc_id = idx.build_from(&term_ids);
    assert_eq!(doc_id, 3);
    term_ids = dict.generate_ids(&vec!["no", "better"]);
    doc_id = idx.build_from(&term_ids);
    assert_eq!(doc_id, 4);
    term_ids = dict.generate_ids(&vec!["well", "sir"]);
    doc_id = idx.build_from(&term_ids);
    assert_eq!(doc_id, 5);
    assert!(idx.is_valid_doc_id(0) == false);
    let term_ids = vec![3, 4]; //vec!["quarrel", "sir"];
    let docs = idx.rank_bm25(&term_ids);
    assert_eq!(docs.len(), 4);
    assert_eq!(term_ids.len(),2); // unchanged
    // DocumentID 2    1    5    3
    // Score      1.98 1.86 0.44 0.18
    let epsilon = 0.005;
    assert_eq!( docs[0].docid, 2 );
    assert!( (docs[0].score - 1.98).abs() <= epsilon );
    assert_eq!( docs[1].docid, 1 );
    assert!( (docs[1].score - 1.86).abs() <= epsilon );
    assert_eq!( docs[2].docid, 5 );
    assert!( (docs[2].score - 0.44).abs() <= epsilon );
    assert_eq!( docs[3].docid, 3 );
    assert!( (docs[3].score - 0.18).abs() <= epsilon );

}

#[test]
fn test_rank_lmd() {
    let mut idx = PositionList::new();
    use super::dictionary::{Dictionary};
    let mut dict = Dictionary::new();
    let mut term_ids = dict.generate_ids(&vec!["do", "you", "quarrel", "sir"]);
    let mut doc_id = idx.build_from(&term_ids);
    assert_eq!(doc_id, 1);
    term_ids = dict.generate_ids(&vec!["quarrel", "sir", "no", "sir"]);
    doc_id = idx.build_from(&term_ids);
    assert_eq!(doc_id, 2);
    term_ids = dict.generate_ids(&vec!["if", "you", "do", "sir", "i", "am", "for", "you", 
    "i", "serve", "as", "good", "a", "man", "as", "you"]);
    doc_id = idx.build_from(&term_ids);
    assert_eq!(doc_id, 3);
    term_ids = dict.generate_ids(&vec!["no", "better"]);
    doc_id = idx.build_from(&term_ids);
    assert_eq!(doc_id, 4);
    term_ids = dict.generate_ids(&vec!["well", "sir"]);
    doc_id = idx.build_from(&term_ids);
    assert_eq!(doc_id, 5);
    assert!(idx.is_valid_doc_id(0) == false);
    let term_ids = vec![3, 4]; //vec!["quarrel", "sir"];
    let docs = idx.rank_lmd(&term_ids);
    assert_eq!(docs.len(), 4);
    assert_eq!(term_ids.len(),2); // unchanged
    // doc 1 score should be 1.25
    let docs_subset:Vec<DocScore> = docs.into_iter().filter(|doc| doc.docid == 1).collect();
    let epsilon = 0.005;
    assert!((docs_subset[0].score - 1.25).abs() < epsilon);
}