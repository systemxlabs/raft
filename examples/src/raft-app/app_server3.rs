use std::sync::{Arc, Mutex};

fn main () {
    println!("Hello, world!");

    // 启动实例3
    let peers = vec![
        raft::peer::Peer::new(1, "http://[::1]:9001".to_string()),
        raft::peer::Peer::new(2, "http://[::1]:9002".to_string()),
    ];
    let consensus: Arc<Mutex<raft::consensus::Consensus>> = raft::start(3, 9003, peers);

    // consensus.lock().unwrap().replicate("hello".to_string());
    loop {
        
    }
}