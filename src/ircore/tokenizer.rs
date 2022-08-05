use unicode_segmentation::UnicodeSegmentation;

pub fn parse_tokens(text: &str) -> Vec<&str>{
    text.unicode_words().collect()
}

pub fn normalize(text: &str) -> String {
    text.to_lowercase()
}

#[test]
fn test_parse_tokens() {
    let text = "Quarrel sir! no, sir!";
    let normalized = normalize(text);
    let tokens = parse_tokens(&normalized);
    assert_eq!(tokens, vec!["quarrel", "sir", "no", "sir"]);
}
