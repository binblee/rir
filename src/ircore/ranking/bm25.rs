use super::super::index::{PositionList, SchemaDependIndex};
use super::super::common::{TermId};
use super::DocScore;
use std::collections::HashMap;

pub trait OkapiBm25 {
    fn rank_bm25(&self, term_ids: &Vec<TermId>) -> Vec<DocScore>;
}

impl OkapiBm25 for PositionList {
    // The BM25 algorithm
    // for all term t sum qt * ftd*(k1+1)/(k1*(1-b+b*(ld/lvag)) + ftd) * log(N/Nt)
    //   qt: query term frequency
    //   ftd: inverted term document frequency (document_frequency[doc_id])
    //   k1: weight saturation factor, default 1.2
    //   b: level of normalization of document length, default 0.75
    //   N: total count of document (term_frequency[term_id, doc_id])
    //   Nt: total count of document that contain term t (document_count)
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
        let document_count = self.get_document_count() as f32;
        let lavg = self.get_average_document_length();
        let docs_contain_any = self.docs_contain_any(&term_ids);
        for docid in docs_contain_any {
            assert!(self.is_valid_doc_id(docid));
            let ld = self.get_document_length(docid) as f32;
            let k1_b_ld_lavg = k1*(1.0-b+b*(ld/lavg));
            let mut score = 0f32;
            for &tid in query_term_freq.keys() {
                let qt = *query_term_freq.get(&tid).unwrap() as f32;
                if let Some(ftd_ref) = self.get_term_frequency(tid, docid){
                    let nt = *self.get_document_frequency(tid).unwrap() as f32;
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::super::index::{PositionList, SchemaDependIndex};
    use super::super::super::dictionary::Dictionary;

    #[test]
    fn test_rank_bm25(){
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
}