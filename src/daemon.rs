
use daemonize::Daemonize;
use std::fs::File;
use std::io::prelude::*;
use std::{thread, time};

pub fn new() {
    let stdout = File::create("/tmp/daemon.out").unwrap();
    let stderr = File::create("/tmp/daemon.err").unwrap();

    let daemonize = Daemonize::new()
        .user("budd")
        .group("budd")
        .stdout(stdout)
        .stderr(stderr);


    let mut n = 1;
    match daemonize.start() {
        
        Ok(_) => loop {
            
            let sleep_time = time::Duration::from_millis(3 * 1000);
            let date = format!("UTC now is: {}\n", n);
            println!("{}",date);
            thread::sleep(sleep_time);
            n = n+ 1;
        },
        Err(e) => eprintln!("Error, {}", e),
    }
}