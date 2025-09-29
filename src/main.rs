use std::{env::args, path::Path};
use heapless::Vec;


mod application;

fn main() {
    println!("Hello, world!");
}

#[test]
fn test_workflow_update_from_local() {
    let path: String = todo!("D");
    let scanner = Scanner::into_path(path).with_ext(MediaExt.ALL);
    let slice: Vec<MediaFile, 1024> = scanner.scan(MediaExt.ALL);

}

fn get_slice(scanner: &mut Scanner) -> (Vec<MediaFile, 1024>, bool) {
    scanner.scan
}