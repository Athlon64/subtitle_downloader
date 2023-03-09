extern crate subdown_rust;

use std::env;
use std::process;

fn main() {
    let files = env::args().skip(1);
    if files.len() == 0 {
        println!("subdown [电影文件名称...]");
    } else {
        for file in files {
            if let Err(e) = subdown_rust::down_sub(file) {
                println!("糟糕，出错了 :(\n{}", e);
                process::exit(1);
            }
        }
    }
}
