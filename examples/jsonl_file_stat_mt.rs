use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::mpsc::channel;

use jsonstat::JsonStat;
use threadpool::ThreadPool;

fn main() {
    if env::args().len() < 2 {
        println!("Usage: jsonl_file_stat <file>");
        return;
    }
    let pool = ThreadPool::new(12);
    let (tx, rx) = channel();
    for file in env::args().skip(1) {
        let _tx = tx.clone();
        pool.execute(move || {
            let f = File::open(file).expect("file open error");
            let mut stat = JsonStat::new();
            for line in BufReader::new(f).lines() {
                let data = line.expect("file read error");
                stat.stat_str(&data);
            }
            _tx.send(stat).expect("send error");
        });
    }
    drop(tx);
    let mut stat = JsonStat::new();
    for s in rx {
        stat.merge(&s);
    }
    println!("{}", stat);
}
