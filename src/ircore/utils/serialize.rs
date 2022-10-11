use std::path::Path;
use std::fs::{self, File};
use std::io::{self, Write, Read};
use serde::{Serialize, Deserialize};


pub fn write_file<T: Serialize>(filepath: &Path, obj: &T) -> io::Result<()>{
    if let Some(dir) = filepath.parent() {
        fs::create_dir_all(dir)?;
    }
    let encoded: Vec<u8> = bincode::serialize(obj).unwrap();
    let mut writer = File::create(filepath)?;
    writer.write_all(&encoded)?;
    Ok(())
}

pub fn read_file<'a, T>(filepath: &Path, encoded: &'a mut Vec<u8>) -> io::Result<T>
    where T: Deserialize<'a> {
    let mut reader = File::open(filepath).expect("cannot open file.");
    match reader.read_to_end(encoded){
        Ok(_) => {
            let reloaded_obj: T = bincode::deserialize(&encoded[..]).unwrap();
            return Ok(reloaded_obj);    
        },
        Err(e) => return Err(e),
    }
}