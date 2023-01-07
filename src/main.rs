mod parse;
use clap::Parser;
use parse::NcmFile;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(author="TexasOct", version="v0.7.0",
about="Hello!", long_about="This is a ncm-flac converter")]

pub struct Args {
    #[arg(short, long, help="src files")]
    files: std::path::PathBuf,

    #[arg(short, long, help="dest directory", default_value="./")]
    output: std::path::PathBuf,
}


fn main() {
    let args = Args::parse();
    let now = Instant::now();
    let mut reader = NcmFile::parse(args.files, args.output);
    reader.output().expect("test");
    let end = now.elapsed();
    println!("time spend: {} millis", end.as_millis());
}