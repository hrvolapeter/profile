use futures::executor::block_on;

use std::thread;

const URL_PATH: &str = "http://speedtest.wdc01.softlayer.com/downloads/test1000.zip";
static NTHREADS: i32 = 8;

pub fn run() {
    for _ in 0..5 {
        let mut children = vec![];
        for _ in 0..NTHREADS {
            children.push(thread::spawn(move || {
                let req = reqwest::get(URL_PATH);
                block_on(req).unwrap();
            }));
        }
        for child in children {
            let _ = child.join();
        }
    }
}
