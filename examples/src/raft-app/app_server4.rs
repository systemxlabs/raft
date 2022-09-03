use std::sync::{Arc, Mutex};
use std::io::Write;

#[derive(Debug)]
struct MyStateMachine {
    datas: Vec<Vec<u8>>,
}
impl raft::state_machine::StateMachine for MyStateMachine {
    fn apply(&mut self, data: &Vec<u8>) {
        self.datas.push(data.clone());
    }
    fn take_snapshot(&mut self, snapshot_filepath: String) {
        let mut snapshot_file = std::fs::File::create(snapshot_filepath.clone()).unwrap();
        let snapshot_json = serde_json::to_string(&self.datas).unwrap();
        if let Err(e) = snapshot_file.write(snapshot_json.as_bytes()) {
            panic!("failed to write snapshot file, error: {}", e)
        }
    }
    fn restore_snapshot(&mut self, snapshot_filepath: String) {
    }
}

fn main () {
    println!("Hello, world!");

    // 启动实例4
    let peers = vec![];
    let state_machine = Box::new(MyStateMachine { datas: Vec::new() });
    let snapshot_dir = format!("{}/{}", std::env::current_dir().unwrap().to_str().unwrap(), ".snapshot/app_server4");
    let metadata_dir = format!("{}/{}", std::env::current_dir().unwrap().to_str().unwrap(), ".metadata/app_server4");
    let consensus: Arc<Mutex<raft::consensus::Consensus>> = raft::start(4, 9004, peers, state_machine, snapshot_dir, metadata_dir);

    let mut count = 0;
    loop {
        std::thread::sleep(std::time::Duration::from_secs(20));
        count += 1;

        let mut consensus = consensus.lock().unwrap();
        // if consensus.state == raft::consensus::State::Leader {
            // consensus.replicate(raft::proto::EntryType::Data, format!("{}", count).as_bytes().to_vec());
        // }
        println!("consensus details: {:#?}", consensus);
    }
}