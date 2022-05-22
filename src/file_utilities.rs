use std::{path::{Path, PathBuf}, fs::{read_to_string, self}, io::{Error, ErrorKind, self, BufRead}, str::FromStr};

use glob::{glob, GlobError};



pub fn create_absolute_path_from_relative(from: &str, relative_path: &str) -> PathBuf {
    let mut path = PathBuf::new();

    path.push(from);
    path.push(relative_path);
    path
}

pub fn get_files_from_wildcard(wildcard: &str, abs_path: &Path) -> Result<Vec<PathBuf>, GlobError> {
    assert!(Path::is_absolute(&abs_path));

    // let glob_pattern = glob(format!("{}/{}", abs_path.to_str().unwrap(), wildcard).as_str());
    
    let wildcarded_path = format!("{}/{}", abs_path.to_str().unwrap(), wildcard);
    let glob_pattern = glob(&wildcarded_path);
    let mut files:Vec<PathBuf> = Vec::new();

    for entry in glob_pattern.unwrap().filter_map(Result::ok) {
        files.push(entry);
    }

    Ok(files)
}

pub fn get_contents(path: PathBuf) -> std::io::Result<String> {
    Ok(read_to_string(path)?)
}

pub fn get_files_from_directory(dir: &dyn AsRef<Path>) -> std::io::Result<Vec<PathBuf>> {
    match fs::read_dir(dir)?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, std::io::Error>>() {
        Ok(it) => Ok(it),
        Err(err) => return Err(err),
    }   
}

fn find_version_annotation(input: &str) -> Option<usize> {
    for line in input.lines() {
        if let Some(0) = line.trim().find("@version") {
            return Some(0);
        }
    }
    None
}

pub fn get_main_config_file(current_dir: &dyn AsRef<Path>) -> std::io::Result<PathBuf> {
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

pub fn get_block_by_position(path_buffer: PathBuf, line_num: u32) -> Option<String> {
    let contents = get_contents(path_buffer);
    let mut buf = vec![];

    if let Ok(contents) = contents {
        let line = contents.lines().nth(line_num.try_into().unwrap()).unwrap();
        let mut cursor = io::Cursor::new(line);

        cursor.read_until(b'(', &mut buf).expect("Reading from cursor won't fail");
    }

    if let Ok(block_name) = std::str::from_utf8(&buf).to_owned() {
        let block_name = &mut block_name.to_string();
        if  block_name.pop() == Some('(') {
            return Some(block_name.trim().to_owned());
        }
    }

    None
}

pub fn get_driver_before_position(path_buffer: PathBuf, line_num: u32) -> Option<String> {
    // <object_type> <id> {
    // <driver> (
        let contents = get_contents(path_buffer).unwrap();
        let mut lines = contents.lines();
        let mut contents_before_pos = String::new();
        let mut curr_line_num: u32 = 0;

        while curr_line_num <= line_num {
            let curr_line = lines.next()?;
            curr_line_num += 1;

            contents_before_pos.push_str(&curr_line);
        }

        // find opening brace
        // find opening parantheses
        if let (Some(brace_pos), Some(paren_pos)) = (contents_before_pos.rfind('{'), contents_before_pos.rfind('(')) {
            let driver_name = contents_before_pos[brace_pos+1..paren_pos].trim().trim_end();
            return Some(driver_name.to_owned());
        }

    None
}

mod tests {
    use std::{env, fs::File, io::Write};

    use super::*;

    const BASE_TEMP_DIR: &str  = "sng_lsp_test_tmp_dir/";

    struct TestDir {
        test_dir: PathBuf
    }

    impl TestDir {

        pub fn new(dir: &str) -> TestDir {
            let test_dir = env::temp_dir().join(BASE_TEMP_DIR).join(dir);
            if !test_dir.exists(){
                fs::create_dir_all(test_dir.clone()).unwrap();
            }
            TestDir {test_dir}
        }

        pub fn get_test_dir(&self) -> &PathBuf{
            &self.test_dir
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            if self.test_dir.exists() {
                fs::remove_dir_all(&self.test_dir).unwrap();
            }
        }
    }

    

    #[test]
    fn test_create_absolute_path_from_relative() {
        let from = "/home/user/folder";
        let relative_path = "project/foo";

        let abs_path = create_absolute_path_from_relative(from, relative_path);

        assert_eq!(abs_path.to_str().unwrap(), "/home/user/folder/project/foo");

    }

    fn fill_directory_with_files(dir: &PathBuf, files: Vec<&str>) {

        for file_name in files {
            File::create(dir.join(file_name)).unwrap();
        }
    }

    fn create_file_abs_path_with_content(abs_path: &dyn AsRef<Path>, content: &str) {
        let mut file = File::create(abs_path).unwrap();
        file.write_all(content.as_bytes());
    }

    #[test]
    fn test_get_files_from_wildcard_star() {
       let tmp = TestDir::new("test_get_files_from_wildcard_star");
       let tmp = tmp.get_test_dir();
        
        // create files
        fill_directory_with_files( &tmp, vec!("a.conf", "b.conf", "foobar.txt"));

        let pattern = "*.conf";
        let matching_files = get_files_from_wildcard(pattern, &tmp).unwrap();

        assert_eq!(matching_files.len(), 2);
    }

    #[test]
    fn test_get_files_from_wildcard_question_mark() {
        let tmp = TestDir::new("test_get_files_from_wildcard_question_mark");
        let tmp = tmp.get_test_dir();

        // create files
        fill_directory_with_files( &tmp, vec!("a1.conf", "a2.conf", "a3.txt"));

        let pattern = "a?.conf";
        let matching_files = get_files_from_wildcard(pattern, &tmp).unwrap();

        assert_eq!(matching_files.len(), 2);
    }

    #[test]
    fn test_get_files_from_directory() {
        let tmp = TestDir::new("test_get_files_from_directory");
        let tmp = tmp.get_test_dir();

        // create files
        fill_directory_with_files( &tmp, vec!("a1.conf", "a2.conf", "a3.txt"));

        let files = get_files_from_directory(&tmp).unwrap();

        assert_eq!(files.len(), 3);
    }

    #[test]
    fn test_get_main_config_file_success() {
        let tmp = TestDir::new("test_get_main_config_file_success");
        let tmp = tmp.get_test_dir();
        let file_name = "main.conf";

        create_file_abs_path_with_content(&tmp.clone().join(&file_name), "@version: 3.35");

        let main_conf_file = get_main_config_file(&tmp).unwrap();

        assert_eq!(main_conf_file.file_name().unwrap().to_str().unwrap(), file_name);
    }

    #[test]
    fn test_get_main_config_file_failure() {
        let tmp = TestDir::new("test_get_main_config_file_failure");
        let tmp = tmp.get_test_dir();
        let file_name = "not-main.conf";

        create_file_abs_path_with_content(&tmp.clone().join(&file_name), "foobar");

        let res = get_main_config_file(&tmp);

        assert!(matches!(res, Err(_)));
    }

    #[test]
    fn test_get_block_by_position() {
        let tmp = TestDir::new("test_get_block_by_position");
        let tmp = tmp.get_test_dir();

        let file_name = "snippet.conf";
        let conf_snippet = r###"
        source s_tls {
            network(
                ip(0.0.0.0) port(1999)
                transport("tls")
                tls(
                    key-file("/opt/syslog-ng/etc/syslog-ng/key.d/syslog-ng.key")
                    cert-file("/opt/syslog-ng/etc/syslog-ng/cert.d/syslog-ng.cert")
                    ca-dir("/opt/syslog-ng/etc/syslog-ng/ca.d")
                )
            );
        };
        "###;

        create_file_abs_path_with_content(&tmp.clone().join(&file_name), conf_snippet);

        let file_path = tmp.clone().join(&file_name);
        
        let block_by_pos = get_block_by_position(file_path, 5).unwrap();
        assert_eq!(&block_by_pos, "tls");
    }

    #[test]
    fn test_get_driver_before_position() {
        let tmp = TestDir::new("test_get_driver_before_position");
        let tmp = tmp.get_test_dir();

        let file_name = "snippet.conf";
        let conf_snippet = r###"
        source s_tls {
            network(
                ip(0.0.0.0) port(1999)
                transport("tls")
                tls(
                    key-file("/opt/syslog-ng/etc/syslog-ng/key.d/syslog-ng.key")
                    cert-file("/opt/syslog-ng/etc/syslog-ng/cert.d/syslog-ng.cert")
                    ca-dir("/opt/syslog-ng/etc/syslog-ng/ca.d")
                )
            );
        };
        "###;

        create_file_abs_path_with_content(&tmp.clone().join(&file_name), conf_snippet);

        let file_path = tmp.clone().join(&file_name);
        
        let block_by_pos = get_driver_before_position(file_path, 2).unwrap();
        assert_eq!(&block_by_pos, "network");
    }

}