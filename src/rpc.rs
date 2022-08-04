use tonic::transport::Channel;

use crate::{consensus, proto, timer};
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};
use logging::*;

// RPC Server
pub struct Server {
    pub consensus: Arc<Mutex<consensus::Consensus>>,
}

#[tokio::main]
pub async fn start_server(addr: &str, server: Server) -> Result<(), Box<dyn std::error::Error>> {
    let addr = addr.parse().unwrap();

    info!("Raft server listening on {}", addr);

    tonic::transport::Server::builder()
        .add_service(proto::consensus_rpc_server::ConsensusRpcServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}

#[tonic::async_trait]
impl proto::consensus_rpc_server::ConsensusRpc for Server {
    async fn append_entries(&self, request: tonic::Request<proto::AppendEntriesReq>,) -> Result<tonic::Response<proto::AppendEntriesResp>, tonic::Status> {
        let addr = request.remote_addr().unwrap();
        info!("Handle append entries request from {:?}, request: {:?}", &addr, &request);
        let response = tonic::Response::new(self.consensus.lock().unwrap().handle_append_entries(&request.into_inner()));
        info!("Handle append entries request from {:?}, response: {:?}", &addr, &response);
        Ok(response)
    }

    async fn request_vote(&self, request: tonic::Request<proto::RequestVoteReq>,) -> Result<tonic::Response<proto::RequestVoteResp>, tonic::Status> {
        let addr = request.remote_addr().unwrap();
        info!("Handle request vote request from {:?}, request: {:?}", &addr, &request);
        let response = tonic::Response::new(self.consensus.lock().unwrap().handle_request_vote(&request.into_inner()));
        info!("Handle request vote request from {:?}, response: {:?}", &addr, &response);
        Ok(response)
    }

    async fn install_snapshot(&self, request: tonic::Request<proto::InstallSnapshotReq>,) -> Result<tonic::Response<proto::InstallSnapshotResp>, tonic::Status> {
        let addr = request.remote_addr().unwrap();
        info!("Handle install snapshot request from {:?}, request: {:?}", &addr, &request);
        let response = tonic::Response::new(self.consensus.lock().unwrap().handle_install_snapshot(&request.into_inner()));
        info!("Handle install snapshot request from {:?}, response: {:?}", &addr, &response);
        Ok(response)
    }
}


// RPC Client
#[derive(Debug)]
pub struct Client {}

impl Client {

    pub async fn append_entries(&mut self, req: proto::AppendEntriesReq, addr: String) -> Result<proto::AppendEntriesResp, Box<dyn std::error::Error>> {
        let addr_clone = addr.clone();

        let request = tonic::Request::new(req);
        info!("send rpc append_entries to {}, request: {:?}", &addr_clone, request);

        let mut client = proto::consensus_rpc_client::ConsensusRpcClient::connect(addr).await?;
        let response =client.append_entries(request).await?;
        info!("send rpc append_entries to {}, response: {:?}", &addr_clone, response);

        Ok(response.into_inner())
    }

    pub async fn request_vote(&mut self, req: proto::RequestVoteReq, addr: String) -> Result<proto::RequestVoteResp, Box<dyn std::error::Error>> {
        let addr_clone = addr.clone();

        let request = tonic::Request::new(req);
        info!("send rpc request_vote to {}, request: {:?}", &addr_clone, request);

        let mut client = proto::consensus_rpc_client::ConsensusRpcClient::connect(addr).await?;
        let response = client.request_vote(request).await?;
        info!("send rpc request_vote to {}, response: {:?}", &addr_clone, response);

        Ok(response.into_inner())
    }

}