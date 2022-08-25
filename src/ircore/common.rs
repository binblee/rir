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