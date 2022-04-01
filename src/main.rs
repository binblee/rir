mod ircore;
use ircore::InvertedIndex;
fn main() {
    let mut index = InvertedIndex::new();
    index.insert("spam".to_string(), 1);
    index.all_phrase(&vec!["spam".to_string()]);
}
