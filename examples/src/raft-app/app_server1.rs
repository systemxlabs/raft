use std::sync::{Arc, Mutex};

fn main () {
    println!("Hello, world!");
    let consensus: Arc<Mutex<raft::consensus::Consensus>> = raft::server::start();
    println!("Hello, world2!");
    consensus.lock().unwrap().replicate();
    loop {
        
    }
}