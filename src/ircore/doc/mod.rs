pub mod text;

// use std::collections::Vec;

pub trait Document {
    fn get_content(&self) -> &str;
    fn get_path(&self) -> &str;
}
