pub mod ircore;
use ircore::engine::Engine;
use clap::{Parser, Subcommand, ValueEnum};
use std::io::{self, BufRead};
use ircore::common::RankingAlgorithm;

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
    /// Search
    Search {
        #[clap(value_parser)]
        /// search phrase
        phrase: Option<String>,
        /// ranking algorithm
        #[clap(short, long, value_enum)]
        ranking: Option<SelectRankingAlgorithm>,

    },
    /// Playgound for try sth new
    SandBox,
}

#[derive(Debug, Clone, ValueEnum)]
enum SelectRankingAlgorithm {
    ExactMatch,
    VectorSpaceModel,
    OkapiBM25,
    LMD,
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
        Some(Commands::Search {phrase, ranking}) => 
            command_search(&cli.index_dir, phrase, ranking)
        ,
        Some(Commands::SandBox) => {
            command_sand_box();
        }
        None => {
            command_load_index(&cli.index_dir);
        }
    }
}

fn command_search(index_dir: &str, phrase_option: &Option<String>, ranking_option: &Option<SelectRankingAlgorithm>){
    let engine = Engine::load_from(index_dir);
    println!("index of {} documents loaded",engine.doc_count());
    match phrase_option {
        Some(phrase_str) => exec_query(&engine, &phrase_str, ranking_option),
        None => {
            println!("input phrase");
            let stdin = io::stdin();
            for line_result in stdin.lock().lines() {
                let line = line_result.unwrap();
                exec_query(&engine, &line, ranking_option);
            }    
        }
    }
}

fn exec_query(engine: &Engine, phrase: &str, ranking_option: &Option<SelectRankingAlgorithm>){
    let ranking;
    match ranking_option {
        Some(SelectRankingAlgorithm::ExactMatch) => ranking = RankingAlgorithm::ExactMatch,
        Some(SelectRankingAlgorithm::VectorSpaceModel) => ranking = RankingAlgorithm::VectorSpaceModel,
        Some(SelectRankingAlgorithm::OkapiBM25) => ranking = RankingAlgorithm::OkapiBM25,
        Some(SelectRankingAlgorithm::LMD) => ranking = RankingAlgorithm::LMD,
        None => ranking = RankingAlgorithm::Default,
    }
    let result = engine.exec_query(phrase, ranking);
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

fn command_build_index(corpus_dir: &str, index_dir: &str) -> io::Result<usize>{
    let mut engine = Engine::new();
    let mut count = 0;
    if let Ok(count_res) = engine.build_index_from(corpus_dir){
        count = count_res;
        engine.save_to(index_dir)?;
        stats(&engine);
    }
    Ok(count)
}

fn command_load_index(index_dir: &str){
    let engine = Engine::load_from(index_dir);
    stats(&engine);
}

fn stats(engine: &Engine) {
    let summary = engine.stats();
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

fn command_sand_box() {
}