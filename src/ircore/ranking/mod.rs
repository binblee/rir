pub mod vsm;
pub mod bm25;
pub mod lmd;
pub mod ps;
use vsm::VectorSpaceModel;
use bm25::OkapiBm25;
use lmd::LanguageModelDivergence;
use ps::PhraseMatch;

use crate::ircore::common::DocId;
pub struct DocScore {
    pub docid: DocId,
    pub score: f32,
}

use crate::ircore::common::{TermId, RankingAlgorithm};
use crate::ircore::index::{PositionList};
pub trait Scorer {
    fn score(&self, terms: &Vec<TermId>, ranking: RankingAlgorithm) -> Vec<DocScore>;
}

impl Scorer for PositionList {
    fn score(&self, terms: &Vec<TermId>, ranking: RankingAlgorithm) -> Vec<DocScore> {
        let docs = vec![];
        if terms.len() == 0 {
            return docs;
        }
        let scorer: fn(&PositionList, &Vec<TermId>) -> Vec<DocScore>;
        match ranking {
            RankingAlgorithm::Default => scorer = PositionList::rank_bm25,
            RankingAlgorithm::ExactMatch => scorer = PositionList::search_phrase,
            RankingAlgorithm::VectorSpaceModel => scorer = PositionList::rank_vsm,
            RankingAlgorithm::OkapiBM25 => scorer = PositionList::rank_bm25,  
            RankingAlgorithm::LMD => scorer = PositionList::rank_lmd,       
        }
        let doc_scores = scorer(&self, &terms);
        doc_scores
    }    
}
