use std::sync::{Arc, Mutex};

fn main () {
    println!("Hello, world!");

    // 启动实例1
    let peers = vec![
        raft::peer::Peer::new(2, "http://[::1]:9002".to_string()),
        raft::peer::Peer::new(3, "http://[::1]:9003".to_string()),
    ];
    let consensus: Arc<Mutex<raft::consensus::Consensus>> = raft::start(1, 9001, peers);

    let mut count = 0;
    loop {
        std::thread::sleep(std::time::Duration::from_secs(20));
        count += 1;

        let mut consensus = consensus.lock().unwrap();
        if consensus.state == raft::consensus::State::Leader {
            consensus.replicate(raft::proto::EntryType::Data, format!("{}", count));
        }
        println!("consensus details: {:#?}", consensus);
    }
}