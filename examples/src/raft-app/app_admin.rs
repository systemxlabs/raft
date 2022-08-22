use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>  {
    println!("Hello, world!");

    let mut client = raft::proto::management_rpc_client::ManagementRpcClient::connect("http://[::1]:9001").await?;

    let get_leader_req = tonic::Request::new(raft::proto::GetLeaderReq {});
    let get_leader_resp = client.get_leader(get_leader_req).await?;
    println!("GetLeader response={:?}", get_leader_resp);

    let get_configuration_req = tonic::Request::new(raft::proto::GetConfigurationReq {});
    let get_configuration_resp = client.get_configuration(get_configuration_req).await?;
    println!("GetConfiguration response={:?}", get_configuration_resp);

    Ok(())
}