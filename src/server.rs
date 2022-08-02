
// pub mod proto {
//     tonic::include_proto!("raft");
// }
use crate::proto;

#[derive(Debug, Default)]
pub struct Server {
    server_id: u64,
    server_addr: String,
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

#[tokio::main]
pub async fn start(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let addr = addr.parse().unwrap();
    let server = Server::default();

    println!("Raft server listening on {}", addr);

    tonic::transport::Server::builder()
        .add_service(proto::consensus_rpc_server::ConsensusRpcServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}