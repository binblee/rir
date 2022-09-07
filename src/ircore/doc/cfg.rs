use serde::{Serialize, Deserialize};
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Cfg {
    file_type: String,
    fields: Vec<String>,
}

impl Cfg {
    pub fn new() -> Self {
        Cfg {
            file_type: String::from("text"),
            fields: vec![],
        }
    }
    pub fn from_str(repo_cfg: &str) -> Self {
        if let Ok(cfg) = serde_yaml::from_str(repo_cfg){
            return cfg;
        }
        return Cfg::new();
    }
    pub fn is_json(&self) -> bool {
        self.file_type.to_lowercase().eq("json")
    }

    pub fn get_fields(&self) -> &Vec<String> {
        &self.fields
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yaml_syntax() {
        let cfg_str = 
"file_type: json
fields:
  - id
  - title
  - url
  - content
";
        let cfg:Cfg = Cfg::from_str(cfg_str);
        assert_eq!(cfg, Cfg { file_type: "json".to_string(), 
                fields: vec!["id".to_string(), "title".to_string(), 
                            "url".to_string(), "content".to_string()] });
        assert!(cfg.is_json());
        assert_eq!(cfg.get_fields(), &vec![
            "id".to_string(), "title".to_string(), 
            "url".to_string(), "content".to_string()]);
    }
}