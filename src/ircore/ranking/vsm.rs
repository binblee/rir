use crate::ircore::{TermId};
use crate::ircore::index::pl::{PositionList, SchemaDependIndex};
use crate::ircore::utils::sparse_vector::SparseVectorOp;
use crate::ircore::ranking::DocScore;

pub trait VectorSpaceModel {
    fn rank_vsm(&self, term_ids: &Vec<TermId>) -> Vec<DocScore>;
}

impl VectorSpaceModel for PositionList {
    fn rank_vsm(&self, term_ids: &Vec<TermId>) -> Vec<DocScore> {
        let mut scores = vec![];
        if term_ids.len() == 0 {
            return scores;
        }
        let query_tfidf = self.get_phrase_tfidf_vector(term_ids);
        // go through all documents that contains at least one term
        for doc_id in self.docs_contain_any(&term_ids) {
            if self.is_valid_doc_id(doc_id){
                let doc_tfidf_vec = self.get_doc_tfidf_vector(doc_id);
                let vec_distance = query_tfidf.vec_dot(&doc_tfidf_vec);
                scores.push(DocScore{docid: doc_id, score: vec_distance});
            }
        }
        // sort by socres
        scores.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap().reverse() );
        scores
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ircore::index::pl::{PositionList, SchemaDependIndex};
    use crate::ircore::token::dictionary::Dictionary;

    #[test]
    fn test_vsm() {
        let mut idx = PositionList::new();
        let mut dict = Dictionary::new();
        let mut term_ids = dict.generate_ids(&vec!["do", "you", "quarrel", "sir"]);
        let mut doc_id = idx.add_document(&term_ids);
        assert_eq!(doc_id, 1);
        term_ids = dict.generate_ids(&vec!["quarrel", "sir", "no", "sir"]);
        doc_id = idx.add_document(&term_ids);
        assert_eq!(doc_id, 2);
        term_ids = dict.generate_ids(&vec!["if", "you", "do", "sir", "i", "am", "for", "you", 
        "i", "serve", "as", "good", "a", "man", "as", "you"]);
        doc_id = idx.add_document(&term_ids);
        assert_eq!(doc_id, 3);
        term_ids = dict.generate_ids(&vec!["no", "better"]);
        doc_id = idx.add_document(&term_ids);
        assert_eq!(doc_id, 4);
        term_ids = dict.generate_ids(&vec!["well", "sir"]);
        doc_id = idx.add_document(&term_ids);
        assert_eq!(doc_id, 5);
        assert!(idx.is_valid_doc_id(0) == false);
        let tfidf_ok = idx.compute_tf_idf();
        assert_eq!(tfidf_ok, Ok(()));
        let term_ids = vec![3, 4]; //vec!["quarrel", "sir"];
        let docs = idx.rank_vsm(&term_ids);
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
}