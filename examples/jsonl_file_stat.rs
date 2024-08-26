use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

use jsonstat::JsonStat;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: jsonl_file_stat <file>");
        return;
    }
    let file = args.get(1).unwrap();
    let f = File::open(file).expect("file open error");
    let mut check = JsonStat::new();
    for line in BufReader::new(f).lines() {
        let data = line.expect("file read error");
        check.stat_str(&data);
    }
    println!("{}", check);
}
