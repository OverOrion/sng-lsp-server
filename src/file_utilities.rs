use std::{path::{Path, PathBuf}, fs::{read_to_string, self}, io::{Error, ErrorKind, self, BufRead}, str::FromStr};

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

pub fn get_files_from_directory(dir: &str) -> std::io::Result<Vec<PathBuf>> {
    match fs::read_dir(dir)?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, std::io::Error>>() {
        Ok(it) => Ok(it),
        Err(err) => return Err(err),
    }   
}

fn find_version_annotation(input: &str) -> Option<usize> {
    for line in input.lines() {
        if let Some(0) = line.find("@version") {
            return Some(0);
        }
    }
    None
}

pub fn get_main_config_file(current_dir: &str) -> std::io::Result<PathBuf> {
    let files = get_files_from_directory(current_dir)?;

    for file in files.iter() {
        let main_conf_file = file;
        let contents =  get_contents(file.to_path_buf())?;
        if let Some(_) = find_version_annotation(&contents) {
            return Ok(main_conf_file.to_path_buf());
        }
    }
    Err(Error::new(ErrorKind::NotFound, "Could not find file with @version, make sure one (and only one) file contains it"))
}

pub fn get_driver_by_position(uri: &str, line_num: u32) -> Option<String> {
    let contents = get_contents(PathBuf::from_str(uri).unwrap());
    let mut buf = vec![];

    if let Ok(contents) = contents {
        let line = contents.lines().nth(line_num.try_into().unwrap()).unwrap();
        let mut cursor = io::Cursor::new(line);

        cursor.read_until(b'(', &mut buf).expect("Reading from cursor won't fail");
    }

    if let Ok(driver_name) = std::str::from_utf8(&buf).to_owned() {
        let driver_name = &mut driver_name.to_string();
        if  driver_name.pop() == Some('(') {
            return Some(driver_name.to_owned());
        }
    }

    None
}