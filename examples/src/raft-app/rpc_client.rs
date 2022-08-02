
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = raft::proto::consensus_rpc_client::ConsensusRpcClient::connect("http://[::1]:9001").await?;

    let request = tonic::Request::new(raft::proto::RequestVoteReq {
        term: 1,
        candidate_id: 1,
        last_log_index: 1,
        last_log_term: 1,
    });

    let response = client.request_vote(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}