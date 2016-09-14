extern crate env_logger;
extern crate v8_api;

use std::env;
use std::path;

fn main() {
    env_logger::init().unwrap();

    let header_file_path = if let Some(path) = env::args_os().nth(1) {
        path::PathBuf::from(path)
    } else if let Some(path) = env::var_os("V8_SOURCE") {
        path::PathBuf::from(path).join("include").join("v8.h")
    } else {
        path::Path::new("/usr/include/v8.h").to_path_buf()
    };

    print!("{}", v8_api::read(&header_file_path, &[] as &[&path::Path]));
}
