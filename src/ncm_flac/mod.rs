mod parse;

use parse::NcmFile;
use std::path::PathBuf;

pub fn parse_multiple_files(files: Vec<PathBuf>, output_path: PathBuf, nums_of_threads: usize) {
    let n = files.len();

    let slice_len = if n % nums_of_threads == n {
        1
    } else {
        n / nums_of_threads
    };

    // get the handles of threads
    let handles: Vec<_> = files
        .chunks(slice_len)
        .map(|group| {
            let group = group.to_vec();
            let path = output_path.clone();
            std::thread::spawn(move || {
                for file in group {
                    let mut reader = NcmFile::parse(file, &path);
                    reader.write_out();
                }
            })
        })
        .collect();

    for (_, handle) in handles.into_iter().enumerate() {
        handle.join().expect("threads error")
    }
}
