pub mod index;
pub mod token;
pub mod doc;
pub mod query;
pub mod utils;
pub mod ranking;

pub type TermId = u32;
pub type DocId = u32;
pub type TermOffset = u32;

pub enum RankingAlgorithm {
    Default,
    ExactMatch,
    VectorSpaceModel,
    OkapiBM25,
    LMD,
}

pub const CFG_NAME: &str = ".rircfg";
