use crate::{proto, timer, peer, log, rpc, util, config};
use std::sync::{Arc, Mutex, Weak};
use std::time::{Instant, Duration};
use logging::*;

#[derive(Debug, PartialEq)]
enum State {
    // Unknown,
    Follower,
    Candidate,
    Leader,
}

#[derive(Debug)]
pub struct Consensus {
    server_id: u64,
    server_addr: String,
    current_term: u64,
    state: State,
    pub election_timer: Arc<Mutex<timer::Timer>>,
    pub heartbeat_timer: Arc<Mutex<timer::Timer>>,
    pub snapshot_timer: Arc<Mutex<timer::Timer>>,
    voted_for: u64,
    commit_index: u64,
    last_applied: u64,
    leader_id: u64,
    peer_manager: peer::PeerManager,
    log: log::Log,
    rpc_client: rpc::Client,
    tokio_runtime: tokio::runtime::Runtime,
}

impl Consensus {

    pub fn new(server_id: u64, port: u32, peers: Vec<peer::Peer>) -> Arc<Mutex<Consensus>> {

        let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
        let mut consensus = Consensus { 
            server_id,
            server_addr: format!("[::1]:{}", port),
            current_term: 0,  // 任期从 0 开始单调递增
            state: State::Follower,
            election_timer: Arc::new(Mutex::new(timer::Timer::new("election_timer"))),
            heartbeat_timer: Arc::new(Mutex::new(timer::Timer::new("heartbeat_timer"))),
            snapshot_timer: Arc::new(Mutex::new(timer::Timer::new("snapshot_timer"))),
            voted_for: config::NONE_SERVER_ID,
            commit_index: 0,  // 已提交日志索引，从0开始单调递增
            last_applied: 0,  // 已应用日志索引，从0开始单调递增
            leader_id: config::NONE_SERVER_ID,
            peer_manager: peer::PeerManager::new(),
            log: log::Log::new(1),
            rpc_client: rpc::Client{},
            tokio_runtime,
        };

        // TODO 加载snapshot

        // TODO 初始化其他服务器peers
        consensus.peer_manager.add_peers(peers);

        Arc::new(Mutex::new(consensus))
    }

    // 上层应用请求复制数据
    pub fn replicate(&mut self, data: String) -> Result<(), Box<dyn std::error::Error>> {
        if self.state != State::Leader {
            error!("replicate should be processed by leader");
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "not leader")));
        }
        info!("replicate data: {}", &data);

        // 存入log entry
        self.log.append(self.current_term, vec![(proto::EntryType::Data, data.as_bytes().to_vec())]);

        // TODO 将 log entry 发送到 peer 节点，可以跟随心跳一起发送
        self.append_entries();

        Ok(())
    }

    // 附加日志到其他节点
    fn append_entries(&mut self) {
        // 检查自己是否为leader
        if self.state != State::Leader {
            error!("state is {:?}, can't append entries", self.state);
            return;
        }

        // TODO 并行发送附加日志请求
        let mut peers = self.peer_manager.peers();
        for peer in peers.iter() {
            let prev_log = self.log.prev_entry(self.log.last_index()).unwrap();
            let req = proto::AppendEntriesReq {
                term: self.current_term,
                leader_id: self.server_id,
                prev_log_index: prev_log.index,
                prev_log_term: prev_log.term,  // TODO
                entries: vec![],
                leader_commit: self.commit_index,
            };
            // TODO 发送附加日志请求
            if let Err(_) = self.tokio_runtime.block_on(self.rpc_client.append_entries(req, peer.server_addr.clone())) {
                error!("append entries to {} failed", &peer.server_addr);
            }
            // TODO 更新 peer 的 next_index 和 match_index

        }
    }

    // 请求其他节点投票
    fn request_vote(&mut self) {
        info!("start request vote");
        let mut vote_granted_count = 0;

        // TODO 并行发送投票请求
        let mut peers = self.peer_manager.peers();
        for peer in peers.iter() {
            info!("request vote to {:?}", &peer.server_addr);
            let req = proto::RequestVoteReq {
                term: self.current_term,
                candidate_id: self.server_id,
                last_log_index: self.log.last_index(),
                last_log_term: self.log.last_term(),
            };
            // TODO 发送投票请求
            if let Ok(resp) = self.tokio_runtime.block_on(self.rpc_client.request_vote(req, peer.server_addr.clone())) {
                info!("request vote to {:?} resp: {:?}", &peer.server_addr, &resp);
                // 对方任期比自己大，退为follower
                if resp.term > self.current_term {
                    info!("peer {} has bigger term {} than self {}", &peer.server_addr, &resp.term, self.current_term);
                    self.state = State::Follower;
                    return;
                }

                // 获得一张选票
                if resp.vote_granted {
                    info!("peer {} vote granted", &peer.server_addr);
                    vote_granted_count += 1;
                }

                // 获得多数选票（包括自己），成为leader，立即发送心跳
                if vote_granted_count + 1 > (peers.len() / 2) {
                    info!("become leader");
                    self.state = State::Leader;
                    self.append_entries();
                    return;
                }

            } else {
                error!("request vote to {:?} failed", &peer.server_addr);
            }
        }
    }

    // TODO 安装快照到其他节点
    fn install_snapshot(&self) {
        unimplemented!();
    }

    // 回退状态
    fn step_down(&mut self, new_term: u64) {
        info!("step down to term {}, current term: {}", new_term, self.current_term);
        if new_term < self.current_term {
            error!("step down failed because new term {} is less than current term {}", new_term, self.current_term);
            return;
        }
        if new_term > self.current_term {
            self.state = State::Follower;
            self.current_term = new_term;
            self.voted_for = config::NONE_SERVER_ID;
            self.leader_id = config::NONE_SERVER_ID;
        } else {
            // Leader收到term相同的RPC，也要回退
            self.state = State::Follower;
        }
        
        // 重置选举计时器
        self.election_timer.lock().unwrap().reset(util::rand_election_timeout());
    }

    // 选举成为leader
    fn become_leader(&mut self) {
        if self.state != State::Candidate {
            error!("can't become leader because state is not candidate, current state is {:?}", self.state);
            return;
        }
        self.state = State::Leader;
        self.leader_id = self.server_id;
        self.append_entries();
    }

    pub fn handle_heartbeat_timeout(&mut self) {
        if self.state == State::Leader {
            // 发送心跳并重置计时器
            info!("handle_heartbeat_timeout");
            self.append_entries();
        }
    }

    pub fn handle_election_timeout(&mut self) {
        match self.state {
            State::Leader => {},
            State::Candidate => {
                // 候选者再次发起选举
                info!("start election again");

                self.current_term += 1;  // 任期加1
                self.voted_for = self.server_id;  // 给自己投票

                // TODO 设置下一次选举超时随机值
                self.election_timer.lock().unwrap().reset(util::rand_election_timeout());

                // 发起投票
                self.request_vote();
            },
            State::Follower => {
                // Follwer发起选举
                info!("start election");

                self.state = State::Candidate;  // 转为候选者
                self.current_term += 1;  // 任期加1
                self.voted_for = self.server_id;  // 给自己投票

                // TODO 设置下一次选举超时随机值
                self.election_timer.lock().unwrap().reset(util::rand_election_timeout());

                // 发起投票
                self.request_vote();
            },
            _ => { error!("unexpected state: {:?} when handling election timeout", self.state); },
        }
    }

    pub fn handle_snapshot_timeout(&mut self) {
        // info!("handle_snapshot_timeout");
    }

    pub fn handle_append_entries(&mut self, request: &proto::AppendEntriesReq) -> proto::AppendEntriesResp {
        let refuse_resp = proto::AppendEntriesResp {
            term: self.current_term,
            success: false,
        };
        // 比较任期
        if request.term < self.current_term {
            return refuse_resp;
        }
        self.step_down(request.term);

        match self.state {
            State::Leader => {
                panic!("leader {} receive append entries from {}", self.server_id, request.leader_id);
            }
            State::Candidate => {
                panic!("candidate {} receive append entries from {}", self.server_id, request.leader_id);
            }
            State::Follower => {
            }
            _ => { error!("unknown state when handling append entries: {:?}", self.state); },
        }
        let reply = proto::AppendEntriesResp {
            success: true,
            term: 1,
        };
        reply
    }

    pub fn handle_request_vote(&mut self, request: &proto::RequestVoteReq) -> proto::RequestVoteResp {
        // 更新任期
        // TODO 如果接收到的 RPC 请求或响应中，任期号T > currentTerm，则令 currentTerm = T，并切换为跟随者状态
        if request.term > self.current_term {
            self.step_down(request.term);
        }
        let refuse_reply = proto::RequestVoteResp {
            term: self.current_term,
            vote_granted: false,
        };

        if self.state == State::Leader {
            // TODO
            return refuse_reply;

        } else if self.state == State::Candidate {

            // 候选者拒绝投票
            return refuse_reply;

        } else if self.state == State::Follower {

            // 如果候选者的任期比自己小，则拒绝投票
            if request.term < self.current_term {
                info!("Refuse vote for {} due to candidate's term is smaller", request.candidate_id);
                return refuse_reply;
            }
            
            // 如果已投票给其他候选者，则拒绝投票
            if self.voted_for != config::NONE_SERVER_ID && self.voted_for != request.candidate_id {
                info!("Refuse vote for {} due to already voted for {}", request.candidate_id, self.voted_for);
                return refuse_reply;
            }

            // 投票给候选者
            info!("Agree vote for server id {}", request.candidate_id);
            self.voted_for = request.candidate_id;
            // TODO 重置选举计时器
            self.election_timer.lock().unwrap().reset(util::rand_election_timeout());
            return proto::RequestVoteResp {
                term: self.current_term,
                vote_granted: true,
            };
        }

        error!("state is {:?}, can't handle request vote", self.state);
        refuse_reply
    }

    pub fn handle_install_snapshot(&mut self, request: &proto::InstallSnapshotReq) -> proto::InstallSnapshotResp {
        info!("handle_install_snapshot");
        let reply = proto::InstallSnapshotResp {
            term: 1,
        };
        reply
    }
    
}
