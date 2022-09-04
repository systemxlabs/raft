use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>  {
    println!("Hello, world!");

    let mut client = raft::proto::management_rpc_client::ManagementRpcClient::connect("http://[::1]:9001").await?;

    let set_configuration_req = tonic::Request::new(raft::proto::SetConfigurationReq {
        new_servers: vec![
            raft::proto::Server {
                server_id: 1,
                server_addr: "http://[::1]:9001".to_string(),
            },
            raft::proto::Server {
                server_id: 2,
                server_addr: "http://[::1]:9002".to_string(),
            },
            raft::proto::Server {
                server_id: 3,
                server_addr: "http://[::1]:9003".to_string(),
            },
            raft::proto::Server {
                server_id: 4,
                server_addr: "http://[::1]:9004".to_string(),
            },
        ]
    });
    let set_configuration_resp = client.set_configuration(set_configuration_req).await?;
    println!("SetConfiguration response={:?}", set_configuration_resp);

    Ok(())
}