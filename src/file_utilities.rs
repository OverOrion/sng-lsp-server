use std::{path::{Path, PathBuf}, fs::read_to_string};

use glob::{glob, GlobError};


pub fn create_absolute_path_from_relative(from: &str, relative_path: &str) -> PathBuf {
    let mut path = PathBuf::new();

    path.push(from);
    path.push(relative_path);
    path
}

pub fn get_files_from_wildcard(wildcard: &str, path: &str) -> Result<Vec<PathBuf>, GlobError> {
    let abs_path = Path::new(path);
    assert!(Path::is_absolute(&abs_path));

    let glob_pattern = glob(format!("{}/{}", path, wildcard).as_str());
    let mut files:Vec<PathBuf> = Vec::new();

    for entry in glob_pattern.unwrap() {

        match entry {
            Ok(path) => files.push(path),

            Err(e) => return Err(e),
        }

    }

    Ok(files)
}

pub fn get_contents(path: PathBuf) -> std::io::Result<String> {
    Ok(read_to_string(path)?)
}