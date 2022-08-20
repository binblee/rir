pub mod ircore;
use std::path::Path;
use ircore::engine::Engine;
use clap::{Parser, Subcommand};
use std::io::{self, BufRead};

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
    /// Phrase search
    SearchPhrase {
        #[clap(value_parser)]
        /// search phrase
        phrase: Option<String>,
    },
    /// Vector space model, ranke cosine
    RankCosine {
        #[clap(value_parser)]
        /// search phrase
        phrase: Option<String>,
    },
    /// Probabilistic model: BM25
    RankBM25 {
        #[clap(value_parser)]
        /// search phrase
        phrase: Option<String>,
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
        Some(Commands::SearchPhrase{ phrase }) =>
            match command_search_phrase(&cli.index_dir, phrase){
                Ok(_) => println!(""),
                Err(_) => eprintln!("error in search phrase"),
            },
        Some(Commands::RankCosine{ phrase }) =>
            match command_rank_cosine(&cli.index_dir, phrase){
                Ok(_) => println!(""),
                Err(_) => eprintln!("error in rank cosine"),
            },
        Some(Commands::RankBM25{ phrase }) =>
            match command_rank_bm25(&cli.index_dir, phrase){
                Ok(_) => println!(""),
                Err(_) => eprintln!("error in BM25"),
            },
        None => {
            command_load_index(&cli.index_dir);
        }
    }
}


fn command_build_index(corpus_dir: &str, index_dir: &str) -> io::Result<usize>{
    let mut engine = Engine::new();
    let mut count = 0;
    if let Ok(count_res) = engine.build_index_from(&Path::new(corpus_dir)){
        count = count_res;
        engine.save_to(&Path::new(index_dir))?;
        info(&engine);
    }
    Ok(count)
}

fn command_search_phrase(index_dir: &str, phrase_option: &Option<String>) -> io::Result<()> {
    let engine = Engine::load_from(Path::new(index_dir));
    println!("index of {} documents loaded",engine.doc_count());
    match phrase_option {
        Some(phrase_str) => search_phrase(&engine, &phrase_str),
        None => {
            println!("input phrase");
            let stdin = io::stdin();
            for line_result in stdin.lock().lines() {
                let line = line_result?;
                search_phrase(&engine, &line);
            }    
        }
    }
    Ok(())
}

fn search_phrase(engine: &Engine, phrase: &str){
    let result = engine.search_phrase(phrase);
    let result_len = result.len();
    if result_len > 0 {
        println!("{} results", result_len);
        let mut display = result_len;
        if result_len > 10 {
            println!("top 10:");
            display = 10;
        }
        for (i,doc) in result.into_iter().enumerate().take(display){
            println!("{}:{}", i+1, doc);
        }
    }else{
        println!("no result");
    }            

}
fn command_load_index(index_dir: &str){
    let engine = Engine::load_from(Path::new(index_dir));
    info(&engine);
}

fn command_rank_cosine(index_dir: &str, phrase_option: &Option<String>) -> io::Result<()> {
    let engine = Engine::load_from(Path::new(index_dir));
    println!("index of {} documents loaded",engine.doc_count());
    match phrase_option {
        Some(phrase_str) => rank_cosine(&engine, &phrase_str),
        None => {
            println!("rank cosine, input phrase");
            let stdin = io::stdin();
            for line_result in stdin.lock().lines() {
                let line = line_result?;
                rank_cosine(&engine, &line);
            }    
        }
    }
    Ok(())
}

fn info(engine: &Engine) {
    let summary = engine.summary();
    println!("===Index===");
    println!("total document: {}", summary.index.document_count);
    println!("total length: {}", summary.index.total_document_length);
    println!("average length: {}", summary.index.average_document_length);
    println!("total term count: {}", summary.analyzer.dict.term_count);
    let display_num = 100;
    println!("===Top {} terms===", display_num);
    let mut sum_so_far:f32 = 0.0;
    for (i, (_, term, count)) in summary.index.term_freq.into_iter().enumerate().take(display_num){
        let freq = count as f32 * 100.0 / summary.index.total_document_length as f32;
        sum_so_far += freq;
        println!("{:5}: {}=>{} ({:.3}%, {:.3}%)", i+1, term, count, freq, sum_so_far);
    }
}

fn rank_cosine(engine: &Engine, phrase: &str){
    let result = engine.rank_cosine(phrase);
    let result_len = result.len();
    if result_len > 0 {
        println!("{} results", result_len);
        let mut display = result_len;
        if result_len > 10 {
            println!("top 10:");
            display = 10;
        }
        for (i,doc) in result.into_iter().enumerate().take(display){
            println!("{}:{}", i+1, doc);
        }
    }else{
        println!("no result");
    }
}

fn command_rank_bm25(index_dir: &str, phrase_option: &Option<String>) -> io::Result<()> {
    let engine = Engine::load_from(Path::new(index_dir));
    println!("index of {} documents loaded",engine.doc_count());
    match phrase_option {
        Some(phrase_str) => rank_bm25(&engine, &phrase_str),
        None => {
            println!("rank BM25, input phrase");
            let stdin = io::stdin();
            for line_result in stdin.lock().lines() {
                let line = line_result?;
                rank_bm25(&engine, &line);
            }    
        }
    }
    Ok(())
}

fn rank_bm25(engine: &Engine, phrase: &str){
    let result = engine.rank_bm25(phrase);
    let result_len = result.len();
    if result_len > 0 {
        println!("{} results", result_len);
        let mut display = result_len;
        if result_len > 10 {
            println!("top 10:");
            display = 10;
        }
        for (i,doc) in result.into_iter().enumerate().take(display){
            println!("{}:{}", i+1, doc);
        }
    }else{
        println!("no result");
    }
}

