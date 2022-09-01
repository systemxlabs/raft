use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct MyStateMachine {
    datas: Vec<Vec<u8>>,
}
impl raft::state_machine::StateMachine for MyStateMachine {
    fn apply(&mut self, data: &Vec<u8>) {
        self.datas.push(data.clone());
    }
    fn take_snapshot(&mut self, snapshot_filepath: String) {
    }
    fn restore_snapshot(&mut self, snapshot_filepath: String) {
    }
}

fn main () {
    println!("Hello, world!");

    // 启动实例1
    let peers = vec![
        raft::peer::Peer::new(2, "http://[::1]:9002".to_string()),
        raft::peer::Peer::new(3, "http://[::1]:9003".to_string()),
    ];
    let state_machine = Box::new(MyStateMachine { datas: Vec::new() });
    let consensus: Arc<Mutex<raft::consensus::Consensus>> = raft::start(1, 9001, peers, state_machine, "./app_server1/".to_string());

    let mut count = 0;
    loop {
        std::thread::sleep(std::time::Duration::from_secs(20));
        count += 1;

        let mut consensus = consensus.lock().unwrap();
        if consensus.state == raft::consensus::State::Leader {
            consensus.replicate(raft::proto::EntryType::Data, format!("{}", count).as_bytes().to_vec());
        }
        println!("consensus details: {:#?}", consensus);
    }
}