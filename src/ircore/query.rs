
use super::common::TermId;
use super::analyzer::Analyzer;

pub struct Query {

}

impl Query {
    pub fn parse(phrase: &str, ignore_non_exist_term: bool, analyzer: &Analyzer) -> Vec<TermId> {
        let (term_ids, unknown_terms) = analyzer.parse(phrase);
        if !ignore_non_exist_term && unknown_terms.len() > 0 {
            return vec![];
        }
        term_ids
    }
}