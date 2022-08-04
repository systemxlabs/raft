use crate::{consensus, proto, timer};
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};

pub struct Server {
    consensus: Arc<Mutex<consensus::Consensus>>,
}

// 启动 raft server
pub fn start() -> Arc<Mutex<consensus::Consensus>> {
    // 创建共识模块
    let consensus = consensus::Consensus::new(9001);

    // 启动 rpc server
    let server = Server {
        consensus: consensus.clone(),
    };
    std::thread::spawn(move || {
        if let Err(_) = start_server("[::1]:9001", server) {
            panic!("tonic rpc server started failed");
        }
    });

    // 启动 timer
    let weak_consensus = Arc::downgrade(&consensus);
    consensus.lock().unwrap().heartbeat_timer.lock().unwrap().schedule(Duration::from_secs(3), move || {
        weak_consensus.upgrade().unwrap().lock().unwrap().handle_heartbeat_timeout();
    });
    let weak_consensus = Arc::downgrade(&consensus);
    consensus.lock().unwrap().election_timer.lock().unwrap().schedule(Duration::from_secs(5), move || {
        weak_consensus.upgrade().unwrap().lock().unwrap().handle_election_timeout();
    });
    let weak_consensus = Arc::downgrade(&consensus);
    consensus.lock().unwrap().snapshot_timer.lock().unwrap().schedule(Duration::from_secs(20), move || {
        weak_consensus.upgrade().unwrap().lock().unwrap().handle_snapshot_timeout();
    });


    consensus
}

// 关闭 raft server
pub fn stop() {
    unimplemented!();
}

#[tokio::main]
async fn start_server(addr: &str, server: Server) -> Result<(), Box<dyn std::error::Error>> {
    let addr = addr.parse().unwrap();

    println!("Raft server listening on {}", addr);

    tonic::transport::Server::builder()
        .add_service(proto::consensus_rpc_server::ConsensusRpcServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}

#[tonic::async_trait]
impl proto::consensus_rpc_server::ConsensusRpc for Server {
    async fn append_entries(&self, request: tonic::Request<proto::AppendEntriesReq>,) -> Result<tonic::Response<proto::AppendEntriesResp>, tonic::Status> {
        println!("Got a request from {:?}", request.remote_addr());
        Ok(tonic::Response::new(self.consensus.lock().unwrap().handle_append_entries(&request.into_inner())))
    }

    async fn request_vote(&self, request: tonic::Request<proto::RequestVoteReq>,) -> Result<tonic::Response<proto::RequestVoteResp>, tonic::Status> {
        println!("Got a request from {:?}", request.remote_addr());
        Ok(tonic::Response::new(self.consensus.lock().unwrap().handle_request_vote(&request.into_inner())))
    }

    async fn install_snapshot(&self, request: tonic::Request<proto::InstallSnapshotReq>,) -> Result<tonic::Response<proto::InstallSnapshotResp>, tonic::Status> {
        println!("Got a request from {:?}", request.remote_addr());
        Ok(tonic::Response::new(self.consensus.lock().unwrap().handle_install_snapshot(&request.into_inner())))
    }
}
