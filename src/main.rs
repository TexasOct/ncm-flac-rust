mod ncm_flac;

use clap::Parser;
use std::time::Instant;
use std::path::PathBuf;
use ncm_flac::parse_multiple_files;

#[derive(Parser, Debug)]
#[command(author="TexasOct", version="v1.0.0",
about="Hello!", long_about="This is a ncm_flac converter")]

pub struct Args {
    #[arg(short, long, help="src files", num_args = 1..)]
    files: Vec<PathBuf>,
    #[arg(short, long, help="destination directory", default_value="./")]
    output: PathBuf,
    #[arg(short, long, help="max nums of thread", default_value="8")]
    thread_num: usize
}


fn main() {
    let args= Args::parse();
    let now= Instant::now();
    parse_multiple_files(args.files, args.output, args.thread_num);
    let end= now.elapsed();
    println!("Total time spend: {} micros", end.as_micros());
}

