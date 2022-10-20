use std::path::Path;
use std::fs::{self, File};
use std::io::{self, Write, Read};
use serde::{Serialize, Deserialize};
use bincode::Options;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use flate2::Compression;

pub fn write_file<T: Serialize>(filepath: &Path, obj: &T) -> io::Result<()>{
    if let Some(dir) = filepath.parent() {
        fs::create_dir_all(dir)?;
    }
    let bincode_options = bincode::DefaultOptions::new().with_varint_encoding().allow_trailing_bytes();
    let encoded: Vec<u8> = bincode_options.serialize(obj).unwrap();
    let f = File::create(filepath)?;
    let mut writer = GzEncoder::new(f, Compression::default());
    writer.write_all(&encoded)?;
    Ok(())
}

pub fn read_file<'a, T>(filepath: &Path, encoded: &'a mut Vec<u8>) -> io::Result<T>
    where T: Deserialize<'a> {
    let f = File::open(filepath).expect("cannot open file.");
    let mut reader = GzDecoder::new(f);
    match reader.read_to_end(encoded){
        Ok(_) => {
            let bincode_options = bincode::DefaultOptions::new().with_varint_encoding().allow_trailing_bytes();
            let reloaded_obj: T = bincode_options.deserialize(&encoded[..]).unwrap();
            return Ok(reloaded_obj);    
        },
        Err(e) => return Err(e),
    }
}