use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};
pub extern crate log as logging;
pub extern crate rand;
pub extern crate lazy_static;

mod timer;
pub mod peer;
mod log;
pub mod proto;
pub mod rpc;
pub mod consensus;
pub mod util;
pub mod config;


// 启动 raft server
pub fn start(server_id: u64, port: u32, peers: Vec<peer::Peer>) -> Arc<Mutex<consensus::Consensus>> {
    // TODO 配置日志测试
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    // 创建共识模块
    let consensus = consensus::Consensus::new(server_id, port, peers);

    // 启动 rpc server
    let server = rpc::Server {
        consensus: consensus.clone(),
    };
    std::thread::spawn(move || {
        let addr = format!("[::1]:{}", port);
        if let Err(_) = rpc::start_server(addr.as_str(), server) {
            panic!("tonic rpc server started failed");
        }
    });

    // 启动 timer
    let weak_consensus = Arc::downgrade(&consensus);
    consensus.lock().unwrap().heartbeat_timer.lock().unwrap().schedule(config::HEARTBEAT_INTERVAL, move || {
        // TODO 处理weak引用不存在情况
        weak_consensus.upgrade().unwrap().lock().unwrap().handle_heartbeat_timeout();
    });
    let weak_consensus = Arc::downgrade(&consensus);
    consensus.lock().unwrap().election_timer.lock().unwrap().schedule(util::rand_election_timeout(), move || {
        weak_consensus.upgrade().unwrap().lock().unwrap().handle_election_timeout();
    });
    let weak_consensus = Arc::downgrade(&consensus);
    consensus.lock().unwrap().snapshot_timer.lock().unwrap().schedule(config::SNAPSHOT_INTERVAL, move || {
        weak_consensus.upgrade().unwrap().lock().unwrap().handle_snapshot_timeout();
    });


    consensus
}


// 关闭 raft server
pub fn stop() {
    unimplemented!();
}
