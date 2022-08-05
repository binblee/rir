pub mod ircore;
use std::path::Path;
use ircore::search::Engine;
use std::env;

fn main() {
    let mut arguments = vec![];
    for arg in env::args().skip(1) {
        arguments.push(arg);
    }
    if arguments.len() == 2 {
        let mut engine = Engine::new();
        if let Ok(num) = engine.build_index(&Path::new(&arguments[0])){
            println!("total {} documents indexed", num);
        }else{
            println!("index failed");
        }
        let docs = engine.search_phase(&arguments[1]);
        println!("find {} docs:\n{:?}", docs.len(), docs);
    }
}



