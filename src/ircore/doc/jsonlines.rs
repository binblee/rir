use std::io;
use crate::ircore::doc::Document;
use serde_json::Value;
use crate::ircore::doc::cfg::Cfg;
use std::path::Path;

pub const FILETYPE:&str = "jsonlines";
pub fn parse_jsonlines(path: &Path, text: &str, cfg: &Cfg) -> io::Result<Vec<Document>> {
    let mut docs = vec![];
    for (n,line) in text.lines().enumerate(){
        let path_string = path.to_string_lossy().to_string();
        match serde_json::from_str::<Value>(line){
            Ok(value) => {
                let mut content = String::new();
                for f in cfg.get_fields() {
                    let field_name = f.to_lowercase();
                    match &value[field_name] {
                        Value::String(s) => {
                            content.push_str(s);
                        },
                        _ => (),
                    }
                }
                docs.push(Document::new(content.clone(), format!("{}:{}",path_string,n+1)));    
            },
            Err(e) => log::warn!("{}:{}",path_string, e),

        }    
    }
    Ok(docs)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::ircore::doc::doc_parser::DocParser;

    #[test]
    fn test_parse_jsonlines_string() {
        let cfg_str = 
"file_type: json
fields:
  - id
  - url
  - title
  - text
";
        let cfg:Cfg = Cfg::from_str(cfg_str);
        let text = r#"

        {"id": "1", "url": "https://someurl/1", "title": "line1", "text": "line1 content"}
        {"id": "2", "url": "https://someurl/2", "title": "line2", "text": "line2 content"}

        "#;
        if let Ok(docs) = parse_jsonlines(Path::new("some path"), text, &cfg){
            assert_eq!(docs.len(), 2);
            let c0 = docs[0].get_content();
            assert_eq!(c0, "1https://someurl/1line1line1 content");
            let c1 = docs[1].get_content();
            assert_eq!(c1, "2https://someurl/2line2line2 content");
            assert_eq!(docs[0].get_path(), "some path:3");
            assert_eq!(docs[1].get_path(), "some path:4");
        }
    }

    #[test]
    fn test_load_chinese_jsonlines() {
        let dp = DocParser::new("./sample_corpus/wiki_lines");
        assert_eq!(dp.get_config().get_file_type(), "jsonlines");
        let docs:Vec<Vec<Document>> = dp.docs().collect();
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0].len(), 3);
        assert_eq!(docs[1].len(), 2);
    }

}