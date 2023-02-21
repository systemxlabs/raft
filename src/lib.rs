use logging::*;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing_subscriber::fmt::writer::MakeWriterExt;
pub extern crate lazy_static;
pub extern crate log as logging;
pub extern crate rand;

pub mod config;
pub mod consensus;
mod log;
mod metadata;
pub mod peer;
pub mod proto;
pub mod rpc;
pub mod snapshot;
pub mod state_machine;
mod timer;
pub mod util;

// 启动 raft server
pub fn start(
    server_id: u64,
    port: u32,
    peers: Vec<peer::Peer>,
    state_machine: Box<dyn state_machine::StateMachine>,
    snapshot_dir: String,
    metadata_dir: String,
) -> Arc<Mutex<consensus::Consensus>> {
    // TODO 配置日志测试
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "info");
    }
    let file_appender = tracing_appender::rolling::hourly("./logs", "application.log");
    let all_appenders = file_appender.and(std::io::stdout);
    tracing_subscriber::fmt().with_writer(all_appenders).init();

    // metadata_dir 不存在进行创建
    if !std::path::Path::new(&metadata_dir).exists() {
        if let Err(e) = std::fs::create_dir_all(metadata_dir.clone()) {
            panic!("failed to create metadata dir, e: {}", e);
        }
    }

    // snapshot_dir 不存在进行创建
    if !std::path::Path::new(&snapshot_dir).exists() {
        if let Err(e) = std::fs::create_dir_all(snapshot_dir.clone()) {
            panic!("failed to create snapshot dir, e: {}", e);
        }
    }

    // 创建共识模块
    let consensus = consensus::Consensus::new(
        server_id,
        port,
        peers,
        state_machine,
        snapshot_dir,
        metadata_dir,
    );

    // 启动 rpc server
    let consensus_clone = consensus.clone();
    std::thread::spawn(move || {
        let addr = format!("[::1]:{}", port);
        if let Err(_) = rpc::start_server(addr.as_str(), consensus_clone) {
            panic!("tonic rpc server started failed");
        }
    });

    // 启动 timer
    let weak_consensus = Arc::downgrade(&consensus);
    consensus
        .lock()
        .unwrap()
        .heartbeat_timer
        .lock()
        .unwrap()
        .schedule(config::HEARTBEAT_INTERVAL, move || {
            if let Some(consensus) = weak_consensus.upgrade() {
                consensus.lock().unwrap().handle_heartbeat_timeout();
            } else {
                error!("heartbeat timer can't call callback function due to consensus was dropped");
            }
        });
    let weak_consensus = Arc::downgrade(&consensus);
    consensus
        .lock()
        .unwrap()
        .election_timer
        .lock()
        .unwrap()
        .schedule(util::rand_election_timeout(), move || {
            if let Some(consensus) = weak_consensus.upgrade() {
                consensus.lock().unwrap().handle_election_timeout();
            } else {
                error!("election timer can't call callback function due to consensus was dropped");
            }
        });
    let weak_consensus = Arc::downgrade(&consensus);
    consensus
        .lock()
        .unwrap()
        .snapshot_timer
        .lock()
        .unwrap()
        .schedule(config::SNAPSHOT_INTERVAL, move || {
            if let Some(consensus) = weak_consensus.upgrade() {
                consensus.lock().unwrap().handle_snapshot_timeout();
            } else {
                error!("snapshot timer can't call callback function due to consensus was dropped");
            }
        });

    consensus
}

// 关闭 raft server
pub fn stop() {
    unimplemented!();
}
