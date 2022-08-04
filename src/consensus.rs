use crate::{proto, timer, peer, log};
use std::sync::{Arc, Mutex, Weak};
use std::time::{Instant, Duration};

#[derive(Debug)]
enum State {
    Unknown,
    Follower,
    Candidate,
    Leader,
}

#[derive(Debug)]
pub struct Consensus {
    server_id: u64,
    server_addr: String,
    current_term: u64,
    state: State,
    pub election_timer: Arc<Mutex<timer::Timer>>,
    pub heartbeat_timer: Arc<Mutex<timer::Timer>>,
    pub snapshot_timer: Arc<Mutex<timer::Timer>>,
    voted_for: u64,
    commit_index: u64,
    last_applied: u64,
    peer_manager: peer::PeerManager,
    log: log::Log,
}

impl Consensus {
    pub fn new(port: u32) -> Arc<Mutex<Consensus>> {
        let consensus = Consensus { 
            server_id: 1, 
            server_addr: format!("127.0.0.1:{}", port),
            current_term: 1, 
            state: State::Follower,
            election_timer: Arc::new(Mutex::new(timer::Timer::new("election_timer"))),
            heartbeat_timer: Arc::new(Mutex::new(timer::Timer::new("heartbeat_timer"))),
            snapshot_timer: Arc::new(Mutex::new(timer::Timer::new("snapshot_timer"))),
            voted_for: 0,
            commit_index: 0,
            last_applied: 0,
            peer_manager: peer::PeerManager::new(),
            log: log::Log::new(1),
        };
        Arc::new(Mutex::new(consensus))
    }

    // 上层应用请求复制数据
    pub fn replicate(&self) {
        println!("replicate");
    }

    // 附加日志到其他节点
    fn append_entries() {
        unimplemented!();
    }
    // 请求其他节点投票
    fn request_vote() {
        unimplemented!();
    }
    // 安装快照到其他节点
    fn install_snapshot() {
        unimplemented!();
    }

    pub fn handle_heartbeat_timeout(&mut self) {
        println!("handle_heartbeat_timeout");
    }
    pub fn handle_election_timeout(&mut self) {
        println!("handle_election_timeout at {:?}", Instant::now());
        // TODO 设置下一次选举超时随机值
        self.election_timer.lock().unwrap().reset(Duration::from_secs(15));
    }
    pub fn handle_snapshot_timeout(&mut self) {
        println!("handle_snapshot_timeout");
    }

    pub fn handle_append_entries(&mut self, request: &proto::AppendEntriesReq) -> proto::AppendEntriesResp {
        println!("handle_append_entires");
        let reply = proto::AppendEntriesResp {
            success: true,
            term: 1,
        };
        reply
    }
    pub fn handle_request_vote(&mut self, request: &proto::RequestVoteReq) -> proto::RequestVoteResp {
        println!("handle_request_vote");
        let reply = proto::RequestVoteResp {
            vote_granted: true,
            term: 1,
        };
        reply
    }
    pub fn handle_install_snapshot(&mut self, request: &proto::InstallSnapshotReq) -> proto::InstallSnapshotResp {
        println!("handle_install_snapshot");
        let reply = proto::InstallSnapshotResp {
            term: 1,
        };
        reply
    }
    
}
