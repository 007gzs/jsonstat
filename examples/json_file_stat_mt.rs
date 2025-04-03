use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::mpsc::channel;

use jsonstat::JsonStat;
use threadpool::ThreadPool;

struct PathWalk {
    ext: String,
    paths: Vec<String>,
    pool: ThreadPool,
}
impl PathWalk {
    fn new(args: Vec<String>) -> PathWalk {
        PathWalk {
            ext: args[1].to_lowercase(),
            paths: args.iter().skip(2).map(|s| s.to_string()).collect(),
            pool: ThreadPool::new(num_cpus::get()),
        }
    }
    fn run(&mut self) -> JsonStat {
        let (tx, rx) = channel();

        while !self.paths.is_empty() {
            let path = self.paths.pop().unwrap();
            let p = Path::new(&path);
            if !p.exists() {
                continue;
            }
            if p.is_dir() {
                for entry in p.read_dir().expect("read dir error") {
                    let entry = entry.expect("read dir entry error");
                    let path = entry
                        .path()
                        .to_str()
                        .expect("path to str error")
                        .to_string();
                    self.paths.push(path);
                }
                continue;
            }
            if !p.is_file() {
                continue;
            }
            if !path.to_lowercase().ends_with(&self.ext) {
                continue;
            }
            let _tx = tx.clone();
            self.pool.execute(move || {
                let mut f = File::open(path).expect("file open error");
                let mut stat = JsonStat::new();
                let mut buf = String::new();
                f.read_to_string(&mut buf).expect("file read error");
                stat.stat_str(&buf);
                _tx.send(stat).expect("send error");
            });
        }

        drop(tx);
        let mut stat = JsonStat::new();
        for s in rx {
            stat.merge(&s);
        }
        stat
    }
}
fn main() {
    if env::args().len() < 3 {
        println!("Usage: json_file_stat ext <path...>");
        return;
    }
    let mut path_walk = PathWalk::new(env::args().collect());

    let stat = path_walk.run();

    println!("{}", stat);
}
