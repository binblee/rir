use super::super::common::TermId;
use super::DocScore;
use super::super::index::{PositionList, SchemaDependIndex};
use std::collections::HashMap;

pub trait LanguageModelDivergence {
    fn rank_lmd(&self, terms: &Vec<TermId>) -> Vec<DocScore>;
}

impl LanguageModelDivergence for PositionList {
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
        let document_count = self.get_document_count() as f32; // N
        let lavg = self.get_average_document_length();
        let query_token_num = terms.len() as f32; // n
        let docs_contain_any = self.docs_contain_any(&terms);
        for docid in docs_contain_any {
            let ld = self.get_document_length(docid) as f32;
            let mut score = 0f32;
            for &tid in query_term_freq.keys() {
                let qt = *query_term_freq.get(&tid).unwrap() as f32;
                if let Some(ftd_ref) = self.get_term_frequency(tid, docid){
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
}

#[cfg(test)]
mod tests {
    use super::super::super::index::{PositionList, SchemaDependIndex};
    use super::super::super::dictionary::Dictionary;
    use super::*;

    #[test]
    fn test_rank_lmd() {
        let mut idx = PositionList::new();
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
}