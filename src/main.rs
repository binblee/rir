pub mod ircore;
use std::path::Path;
use ircore::search::Engine;

fn main() {
    let mut engine = Engine::new();
    let res = engine.build_index(&Path::new("./samples"));
    assert_eq!(res, Ok(5));
    let docs = engine.search_phase("Quarrel sir");
    println!("{:?}", docs);
}



