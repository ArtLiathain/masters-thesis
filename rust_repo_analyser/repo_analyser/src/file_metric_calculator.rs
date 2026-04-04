use std::{fs, io::read_to_string, path::PathBuf};

use rust_code_analysis::{action, get_function_spaces, CppParser, Metrics, MetricsCfg};

pub fn create_csv_of_file_metrics(data_path: String) {
    let paths = fs::read_dir(data_path).unwrap();

    for path in paths {
        println!("Name: {}", path.unwrap().path().display());
        let path_buf = PathBuf::from(path);
        let file_as_bytes = fs::read(path.unwrap())
            .map_err(|err| format!("Failed to read file {}", err))
            .unwrap();
        if let Some(space) = get_function_spaces(
            &rust_code_analysis::LANG::Cpp,
            file_as_bytes,
            &path_buf,
            None,
        ) {
            println!("File: {:?}", space.name);
        }
    }
}
