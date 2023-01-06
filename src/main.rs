mod parse;

use std::error::Error;
use clap::Parser;
use parse::NcmFile;

#[derive(Parser, Debug)]
#[command(author="TexasOct", version="v0.7.0",
about="Hello!", long_about="This is a ncm-flac converter")]

pub struct Args {
    #[arg(short, long, help="src files")]
    files: std::path::PathBuf,

    #[arg(short, long, help="dest directory", default_value="./")]
    output: std::path::PathBuf,
}


fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mut reader = NcmFile::parse(args.files, args.output);
    reader.output()
}