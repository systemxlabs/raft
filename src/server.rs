use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use crate::{proto, timer, peer, log};

#[derive(Debug)]
enum ServerState {
    Unknown,
    Follower,
    Candidate,
    Leader,
}

#[derive(Debug)]
pub struct Server {
    server_id: u64,
    server_addr: String,
    current_term: u64,
    state: ServerState,
    election_timer: timer::Timer,
    heartbeat_timer: timer::Timer,
    snapshot_timer: timer::Timer,
    voted_for: u64,
    commit_index: u64,
    last_applied: u64,
    peer_manager: peer::PeerManager,
    log: log::Log,
}

impl Server {
    pub fn new(port: u32) -> Server {
        Server { 
            server_id: 1, 
            server_addr: format!("127.0.0.1:{}", port),
            current_term: 1, 
            state: ServerState::Follower,
            election_timer: timer::Timer::new(),
            heartbeat_timer: timer::Timer::new(),
            snapshot_timer: timer::Timer::new(),
            voted_for: 0,
            commit_index: 0,
            last_applied: 0,
            peer_manager: peer::PeerManager::new(),
            log: log::Log::new(1),
        }
    }

    // 启动 raft server
    fn start() {

    }

    // 关闭 raft server
    fn stop() {

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
    
}

#[tonic::async_trait]
impl proto::consensus_rpc_server::ConsensusRpc for Server {
    async fn append_entries(&self, request: tonic::Request<proto::AppendEntriesReq>,) -> Result<tonic::Response<proto::AppendEntriesResp>, tonic::Status> {
        println!("Got a request from {:?}", request.remote_addr());
        let reply = proto::AppendEntriesResp {
            success: true,
            term: 1,
        };
        Ok(tonic::Response::new(reply))
    }

    async fn request_vote(&self, request: tonic::Request<proto::RequestVoteReq>,) -> Result<tonic::Response<proto::RequestVoteResp>, tonic::Status> {
        println!("Got a request from {:?}", request.remote_addr());
        let reply = proto::RequestVoteResp {
            vote_granted: true,
            term: 1,
        };
        Ok(tonic::Response::new(reply))
    }

    async fn install_snapshot(&self, request: tonic::Request<proto::InstallSnapshotReq>,) -> Result<tonic::Response<proto::InstallSnapshotResp>, tonic::Status> {
        println!("Got a request from {:?}", request.remote_addr());
        let reply = proto::InstallSnapshotResp {
            term: 1,
        };
        Ok(tonic::Response::new(reply))
    }
}


// 启动 raft server
#[tokio::main]
pub async fn start(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let addr = addr.parse().unwrap();
    let server = Server::new(9001);

    println!("Raft server listening on {}", addr);

    tonic::transport::Server::builder()
        .add_service(proto::consensus_rpc_server::ConsensusRpcServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}


// 关闭 raft server
pub fn stop() {
    unimplemented!();
}