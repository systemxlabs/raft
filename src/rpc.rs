use tonic::transport::Channel;

use crate::consensus::Consensus;
use crate::{consensus, proto, timer};
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};
use logging::*;

// RPC Server
pub struct Server {
    pub consensus: Arc<Mutex<consensus::Consensus>>,
}

#[tokio::main]
pub async fn start_server(addr: &str, consensus: Arc<Mutex<Consensus>>) -> Result<(), Box<dyn std::error::Error>> {
    let addr = addr.parse().unwrap();

    info!("Raft server listening on {}", addr);

    let consensus_server = Server {
        consensus: consensus.clone()
    };
    let management_server = Server {
        consensus: consensus.clone()
    };
    tonic::transport::Server::builder()
        .add_service(proto::consensus_rpc_server::ConsensusRpcServer::new(consensus_server))
        .add_service(proto::management_rpc_server::ManagementRpcServer::new(management_server))
        .serve(addr)
        .await?;

    Ok(())
}

#[tonic::async_trait]
impl proto::consensus_rpc_server::ConsensusRpc for Server {
    async fn append_entries(&self, request: tonic::Request<proto::AppendEntriesReq>,) -> Result<tonic::Response<proto::AppendEntriesResp>, tonic::Status> {
        let addr = request.remote_addr().unwrap();
        info!("Handle append entries from {:?}, request: {:?}", &addr, &request);
        let response = tonic::Response::new(self.consensus.lock().unwrap().handle_append_entries(&request.into_inner()));
        info!("Handle append entries from {:?}, response: {:?}", &addr, &response);
        Ok(response)
    }

    async fn request_vote(&self, request: tonic::Request<proto::RequestVoteReq>,) -> Result<tonic::Response<proto::RequestVoteResp>, tonic::Status> {
        let addr = request.remote_addr().unwrap();
        info!("Handle request vote from {:?}, request: {:?}", &addr, &request);
        let response = tonic::Response::new(self.consensus.lock().unwrap().handle_request_vote(&request.into_inner()));
        info!("Handle request vote from {:?}, response: {:?}", &addr, &response);
        Ok(response)
    }

    async fn install_snapshot(&self, request: tonic::Request<proto::InstallSnapshotReq>,) -> Result<tonic::Response<proto::InstallSnapshotResp>, tonic::Status> {
        let addr = request.remote_addr().unwrap();
        info!("Handle install snapshot from {:?}, request: {:?}", &addr, &request);
        let response = tonic::Response::new(self.consensus.lock().unwrap().handle_install_snapshot(&request.into_inner()));
        info!("Handle install snapshot from {:?}, response: {:?}", &addr, &response);
        Ok(response)
    }
}

#[tonic::async_trait]
impl proto::management_rpc_server::ManagementRpc for Server {
    async fn get_leader(&self, request: tonic::Request<proto::GetLeaderReq>) -> Result<tonic::Response<proto::GetLeaderResp>, tonic::Status> {
        let addr = request.remote_addr().unwrap();
        info!("Handle get leader from {:?}, request: {:?}", &addr, &request);
        let response = tonic::Response::new(self.consensus.lock().unwrap().handle_get_leader(&request.into_inner()));
        info!("Handle get leader from {:?}, response: {:?}", &addr, &response);
        Ok(response)
    }

    async fn get_configuration(&self, request: tonic::Request<proto::GetConfigurationReq>) -> Result<tonic::Response<proto::GetConfigurationResp>, tonic::Status> {
        let addr = request.remote_addr().unwrap();
        info!("Handle get configuration from {:?}, request: {:?}", &addr, &request);
        let response = tonic::Response::new(self.consensus.lock().unwrap().handle_get_configuration(&request.into_inner()));
        info!("Handle get configuration from {:?}, response: {:?}", &addr, &response);
        Ok(response)
    }

    async fn set_configuration(&self, request: tonic::Request<proto::SetConfigurationReq>) -> Result<tonic::Response<proto::SetConfigurationResp>, tonic::Status> {
        let addr = request.remote_addr().unwrap();
        info!("Handle set configuration from {:?}, request: {:?}", &addr, &request);
        let response = tonic::Response::new(self.consensus.lock().unwrap().handle_set_configuration(&request.into_inner()));
        info!("Handle set configuration from {:?}, response: {:?}", &addr, &response);
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