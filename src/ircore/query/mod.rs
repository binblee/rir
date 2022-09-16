use crate::ircore::TermId;
use crate::ircore::token::analyzer::Analyzer;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ircore::token::analyzer::Analyzer;

    #[test]
    fn test_query() {
        let mut analyzer = Analyzer::new();
        let terms_add_to_dict = analyzer.analyze("Do you QUARREL, sir?");
        assert_eq!(terms_add_to_dict, vec![1, 2, 3, 4]);
        let phrase_str = "Sir quarrel";
        let terms = Query::parse(phrase_str, true, &analyzer);
        assert_eq!(terms, vec![4, 3]);
    }
}
