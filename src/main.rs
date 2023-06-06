mod parse;

use clap::Parser;
use parse::NcmFile;
use std::time::Instant;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author="TexasOct", version="v1.0.0",
about="Hello!", long_about="This is a ncm-flac converter")]

pub struct Args {
    #[arg(short, long, help="src files")]
    files: PathBuf,
    #[arg(short, long, help="dest directory", default_value="./")]
    output: PathBuf,
}


fn main() {
    let args = Args::parse();
    let now = Instant::now();
    let mut reader = NcmFile::parse(args.files, args.output);
    reader.output().expect("test");
    let end = now.elapsed();
    println!("Total time spend: {} micros", end.as_micros());
}

