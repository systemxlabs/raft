use std::io::{Read, Write};
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
        let mut snapshot_file = std::fs::File::create(snapshot_filepath.clone()).unwrap();
        let snapshot_json = serde_json::to_string(&self.datas).unwrap();
        if let Err(e) = snapshot_file.write(snapshot_json.as_bytes()) {
            panic!("failed to write snapshot file, error: {}", e)
        }
    }
    fn restore_snapshot(&mut self, snapshot_filepath: String) {
        if std::path::Path::new(&snapshot_filepath).exists() {
            let mut snapshot_file = std::fs::File::open(snapshot_filepath).unwrap();
            let mut snapshot_json = String::new();
            snapshot_file
                .read_to_string(&mut snapshot_json)
                .expect("failed to read snapshot");
            let datas: Vec<Vec<u8>> = serde_json::from_str(snapshot_json.as_str()).unwrap();
            self.datas = datas;
        }
    }
}

fn main() {
    println!("Hello, world!");

    // 启动实例2
    let peers = vec![
        raft::peer::Peer::new(1, "http://[::1]:9001".to_string()),
        raft::peer::Peer::new(3, "http://[::1]:9003".to_string()),
    ];
    let state_machine = Box::new(MyStateMachine { datas: Vec::new() });
    let snapshot_dir = format!(
        "{}/{}",
        std::env::current_dir().unwrap().to_str().unwrap(),
        ".snapshot/app_server2"
    );
    let metadata_dir = format!(
        "{}/{}",
        std::env::current_dir().unwrap().to_str().unwrap(),
        ".metadata/app_server2"
    );
    let consensus: Arc<Mutex<raft::consensus::Consensus>> =
        raft::start(2, 9002, peers, state_machine, snapshot_dir, metadata_dir);

    let mut count = 0;
    loop {
        std::thread::sleep(std::time::Duration::from_secs(20));
        count += 1;

        let mut consensus = consensus.lock().unwrap();
        if consensus.state == raft::consensus::State::Leader {
            consensus.replicate(
                raft::proto::EntryType::Data,
                format!("{}", count).as_bytes().to_vec(),
            );
        }
        println!("consensus details: {:#?}", consensus);
    }
}
