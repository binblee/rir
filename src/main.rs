pub mod ircore;
use std::path::Path;
use ircore::search::Engine;
use clap::{Parser, Subcommand};
use std::io;

#[derive(Parser)]
#[derive(Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
/// A simple information retrieval engine
struct Cli {
    #[clap(short, long, value_parser, default_value_t = String::from(".rir/rir.idx"))]
    /// Index directory
    index_dir: String,
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
#[derive(Debug)]
enum Commands {
    /// Build index
    Build {
        #[clap(short, long, value_parser)]
        /// Corpus directory
        corpus_dir: String,
    },
    /// Phase search
    SearchPhase {
        #[clap(value_parser)]
        /// search phase
        phase: String,
    }
}

fn main() {
    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Some(Commands::Build { corpus_dir}) => 
            match command_build_index(corpus_dir, &cli.index_dir){
                Ok(count) => println!("{} documents indexed", count),
                Err(_) => eprintln!("error in processing")
        },
        Some(Commands::SearchPhase{ phase }) =>{
            command_search_phase(&cli.index_dir, phase);
        },
        None => {
            command_load_index(&cli.index_dir);
        }
    }
}


fn command_build_index(corpus_dir: &str, index_dir: &str) -> io::Result<usize>{
    let mut engine = Engine::new();
    let mut count = 0;
    if let Ok(count_res) = engine.build_index(&Path::new(corpus_dir)){
        count = count_res;
        engine.save_to(&Path::new(index_dir))?;
    }
    Ok(count)
}

fn command_search_phase(index_dir: &str, phase: &str) {
    let engine = Engine::load_from(Path::new(index_dir));
    println!("index of {} documents loaded",engine.doc_count());
    let result = engine.search_phase(phase);
    let result_len = result.len();
    if result_len > 0 {
        println!("{} results", result_len);
        let mut display = result_len;
        if result_len > 10 {
            println!("top 10:");
            display = 10;
        }
        for i in 0..display{
            println!("{}:{}", i+1, result[i]);
        }
    }
}

fn command_load_index(index_dir: &str) {
    let engine = Engine::load_from(Path::new(index_dir));
    println!("index of {} documents loaded",engine.doc_count());
}



