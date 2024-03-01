mod parse;

use std::cmp::min;
use std::path::PathBuf;
use parse::NcmFile;

pub fn parse_multiple_files(files: Vec<PathBuf>, output_path: PathBuf, nums_of_threads: usize) {
    let n = files.len();

    let num = if n % nums_of_threads == n {
        1
    } else {
        n / nums_of_threads
    };


    let mut handles = Vec::new();

    let mut flag = 0;

    while flag < n {
        let group = files[flag ..min(flag + num, n)].to_vec();
        let path = output_path.clone();
        handles.push(std::thread::spawn( move || {
            let inner_path = path;
            for file in group {
                let mut reader = NcmFile::parse(file, inner_path.clone());
                reader.output().unwrap();
            }
        }));
        flag += num;
    }


    for (index, handle) in handles.into_iter().enumerate() {
        println!("start handle {:}", index);
        handle.join().expect("threads error")
    }
}