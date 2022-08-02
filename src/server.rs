use crate::{consensus, proto};
use std::sync::{Arc, Mutex};

pub struct Server {
    consensus: Arc<Mutex<consensus::Consensus>>,
}

// 启动 raft server
pub fn start() -> Arc<Mutex<consensus::Consensus>> {
    let consensus = consensus::Consensus::new(9001);
    let server = Server {
        consensus: consensus.clone(),
    };
    std::thread::spawn(move || {
        start_server("[::1]:9001", server);
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
