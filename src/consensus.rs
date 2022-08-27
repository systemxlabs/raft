use crate::{proto, timer, peer, log, rpc, util, config, state_machine};
use std::sync::{Arc, Mutex, Weak};
use std::time::{Instant, Duration};
use std::cell::RefCell;
use logging::*;

#[derive(Debug, PartialEq)]
pub enum State {
    Follower,
    Candidate,
    Leader,
}

#[derive(Debug)]
pub struct Consensus {
    pub server_id: u64,
    pub server_addr: String,
    pub current_term: u64,  // TODO 持久性状态
    pub state: State,
    pub voted_for: u64,  // TODO 持久性状态
    pub commit_index: u64,
    pub last_applied: u64,
    pub leader_id: u64,
    pub peer_manager: peer::PeerManager,
    pub log: log::Log,  // TODO 持久性状态
    pub configuration_state: config::ConfigurationState,
    pub election_timer: Arc<Mutex<timer::Timer>>,
    pub heartbeat_timer: Arc<Mutex<timer::Timer>>,
    pub snapshot_timer: Arc<Mutex<timer::Timer>>,
    rpc_client: rpc::Client,  // TODO 可以将RPC Client移到peer中
    tokio_runtime: tokio::runtime::Runtime,
    pub state_machine: Box<dyn state_machine::StateMachine>,
}

impl Consensus {

    pub fn new(server_id: u64, port: u32, peers: Vec<peer::Peer>, state_machine: Box<dyn state_machine::StateMachine>) -> Arc<Mutex<Consensus>> {

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
            configuration_state: config::ConfigurationState::new(),
            rpc_client: rpc::Client{},
            tokio_runtime,
            state_machine,
        };

        // TODO 加载snapshot

        // TODO 初始化其他服务器peers
        consensus.peer_manager.add_peers(peers);

        Arc::new(Mutex::new(consensus))
    }

    // 上层应用请求复制数据
    pub fn replicate(&mut self, r#type: proto::EntryType, data: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        if self.state != State::Leader {
            error!("replicate should be processed by leader");
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "not leader")));
        }
        info!("replicate data: {:?}", &data);

        // 存入 log entry
        self.log.append_data(self.current_term, vec![(r#type, data.clone())]);

        // 立刻应用Configuration条目
        if r#type == proto::EntryType::Configuration {
            self.apply_configuration(config::Configuration::from_data(&data));
        }

        // 将 log entry 发送到 peer 节点
        self.append_entries(false);

        // TODO 在条目被应用到状态机后响应客户端

        Ok(())
    }

    // 附加日志到其他节点
    fn append_entries(&mut self, heartbeat: bool) -> bool {
        // 检查自己是否为leader
        if self.state != State::Leader {
            error!("state is {:?}, can't append entries", self.state);
            return false;
        }

        // TODO 并行发送附加日志请求
        let peer_server_ids = self.peer_manager.peer_server_ids();
        if peer_server_ids.is_empty() {
            self.leader_advance_commit_index();
        }
        for peer_server_id in peer_server_ids.iter() {
            self.append_entries_to_peer(peer_server_id.clone(), heartbeat);
        }

        true
    }

    fn append_entries_to_peer(&mut self, peer_server_id: u64, heartbeat: bool) -> bool {
        let peer = self.peer_manager.peer(peer_server_id).unwrap();
        let entries: Vec<proto::LogEntry> = match heartbeat {
            true => {self.log.pack_entries(peer.next_index)},
            false => {Vec::new()}
        };
        let entries_num = entries.len();

        let prev_log_index = peer.next_index - 1;
        let prev_log = self.log.entry(prev_log_index).unwrap();
        let req = proto::AppendEntriesReq {
            term: self.current_term,
            leader_id: self.server_id,
            prev_log_index: prev_log_index,
            prev_log_term: prev_log.term,
            entries,
            leader_commit: self.commit_index,
        };
        // 发送附加日志请求
        let resp = match self.tokio_runtime.block_on(self.rpc_client.append_entries(req, peer.server_addr.clone())) {
            Ok(resp) => {resp},
            Err(_) => {
                error!("append entries to {} failed", &peer.server_addr);
                return false;},
        };

        // 比较任期
        if resp.term > self.current_term {
            self.step_down(resp.term);
            return false;
        }

        match resp.success {
            true => {
                // 更新 next_index 和 match_index
                peer.match_index = prev_log_index + entries_num as u64;
                peer.next_index = peer.match_index + 1;
                self.leader_advance_commit_index();
                return true;
            }
            false => {
                // next_index减一
                if peer.next_index > 1 {
                    peer.next_index -= 1;
                }
                return false;
            }
        }

        true
    }

    // 请求其他节点投票
    fn request_vote(&mut self) {
        info!("start request vote");

        // 重置peer选票信息
        self.peer_manager.reset_vote();

        // TODO 并行发送投票请求
        let peer_server_ids = self.peer_manager.peer_server_ids();
        for peer_server_id in peer_server_ids.iter() {
            let peer = self.peer_manager.peer(peer_server_id.clone()).unwrap();
            info!("request vote to {:?}", &peer.server_addr);
            let req = proto::RequestVoteReq {
                term: self.current_term,
                candidate_id: self.server_id,
                last_log_index: self.log.last_index(),
                last_log_term: self.log.last_term(),
            };

            // 发送投票请求
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
                    peer.vote_granted = true;
                }

                // 获得多数选票（包括自己），成为leader，立即发送心跳
                if self.peer_manager.quorum_vote_granted(&self.configuration_state) {
                    info!("become leader");
                    self.become_leader();
                    return;
                }

            } else {
                error!("request vote to {:?} failed", &peer.server_addr);
            }
        }

        // 获得多数选票（包括自己），成为leader，立即发送心跳
        if self.peer_manager.quorum_vote_granted(&self.configuration_state) {
            info!("become leader");
            self.become_leader();
            return;
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
        self.state = State::Follower;
        if new_term > self.current_term {
            self.current_term = new_term;
            self.voted_for = config::NONE_SERVER_ID;
            self.leader_id = config::NONE_SERVER_ID;
        } else {
            // new_term == self.current_term 情况
            // 1.Leader收到AppendEntries RPC => 回退到Follower
            // 2.Leader收到RequestVote RPC => 不回退
            // 3.Candidate收到AppendEntries RPC => 回退到Follower
            // 4.Candidate收到RequestVote RPC => 不回退
            self.leader_id = config::NONE_SERVER_ID;
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
        // 添加NOOP日志
        if let Err(e) = self.replicate(proto::EntryType::Noop, config::NONE_DATA.as_bytes().to_vec()) {
            error!("add noop entry failed after becoming leader, error: {:?}", e);
        }
    }

    fn leader_advance_commit_index(&mut self) {
        let new_commit_index = self.peer_manager.quorum_match_index(&self.configuration_state, self.log.last_index());
        if new_commit_index <= self.commit_index {
            return;
        }
        info!("advance commit index from {} to {}", self.commit_index, new_commit_index);
        let prev_commit_index = self.commit_index;

        for index in prev_commit_index + 1 .. new_commit_index + 1 {
            let entry = self.log.entry(index).unwrap();
            match entry.r#type() {
                // 应用到状态机
                proto::EntryType::Data => {
                    info!("apply data entry: {:?}", entry);
                    self.state_machine.apply(&entry.data);
                    self.commit_index += 1;
                },
                // 新增Cnew配置条目
                proto::EntryType::Configuration => {
                    self.commit_index += 1;
                    let configuration = config::Configuration::from_data(&entry.data);
                    if !configuration.old_servers.is_empty() && !configuration.new_servers.is_empty() {
                        info!("append Cnew entry when Cold,new commited, Cold,new: {:?}", &configuration);
                        self.append_configuration(None);
                    }
                },
                proto::EntryType::Noop => {
                    self.commit_index += 1;
                },
            }
        }
    }

    fn follower_advance_commit_index(&mut self, leader_commit_index: u64) {
        if self.commit_index < leader_commit_index {
            info!("follower advance commit index from {} to {}", self.commit_index, leader_commit_index);
            
            let prev_commit_inndex = self.commit_index;
            // 应用到状态机
            for index in prev_commit_inndex + 1 .. leader_commit_index + 1 {
                if let Some(entry) = self.log.entry(index) {
                    if entry.r#type() == proto::EntryType::Data {
                        info!("apply data entry: {:?}", entry);
                        self.state_machine.apply(&entry.data);
                    }
                    self.commit_index += 1;
                } else {
                    // 可能日志还没catchup
                    break;
                }

            }
        }
    }

    // 应用Configuration条目
    fn apply_configuration(&mut self, configuration: config::Configuration) {
        // 更新peers列表
        let mut new_peers = Vec::new();
        for server_info in configuration.new_servers.iter() {
            if !self.peer_manager.contains(server_info.0) && server_info.0 != self.server_id {
                new_peers.push(peer::Peer::new(server_info.0, server_info.1.clone()));
            }
        }
        self.peer_manager.add_peers(new_peers);

        // TODO 下线？
        
        // 更新节点配置状态
        for peer in self.peer_manager.peers_mut().iter_mut() {
            peer.configuration_state = configuration.query_configuration_state(peer.server_id);
        }
        self.configuration_state = configuration.query_configuration_state(self.server_id);
    }

    // 添加Configuration日志条目
    fn append_configuration(&mut self, new_servers: Option<&Vec<proto::Server>>) -> bool {
        match new_servers {
            // 添加Cold,new日志条目
            Some(servers) => {
                let mut old_new_configuration = config::Configuration::new();
                old_new_configuration.append_new_servers(servers);
                old_new_configuration.append_old_peers(self.peer_manager.peers());
                old_new_configuration.old_servers.push(config::ServerInfo(self.server_id, self.server_addr.clone()));

                match self.replicate(proto::EntryType::Configuration, old_new_configuration.to_data()) {
                    Ok(_) => { return true; },
                    Err(_) => { return false; },
                };
            },

            // 添加Cnew日志条目
            None => {
                let old_new_configuration = self.log.last_configuration();
                if old_new_configuration.old_servers.is_empty() || old_new_configuration.new_servers.is_empty() {
                    panic!("There is no Cold,new before when appending Cnew");
                }
                let new_configuration = old_new_configuration.gen_new_configuration();

                match self.replicate(proto::EntryType::Configuration, new_configuration.to_data()) {
                    Ok(_) => { return true; },
                    Err(_) => {return false; },
                };
            }
        }
    }

    pub fn handle_heartbeat_timeout(&mut self) {
        if self.state == State::Leader {
            // 发送心跳
            info!("handle_heartbeat_timeout");
            self.append_entries(true);
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

                // 重置选举计时器
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

                // 重置选举计时器
                self.election_timer.lock().unwrap().reset(util::rand_election_timeout());

                // 发起投票
                self.request_vote();
            },
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

        // leader或candidate 收到term不小于自己的leader的AppendEntries RPC => 回退到Follower
        self.step_down(request.term);

        // 更新leaderid
        if self.leader_id == config::NONE_SERVER_ID {
            info!("update leader id to {}", request.leader_id);
            self.leader_id = request.leader_id;
        }
        if self.leader_id != request.leader_id {
            error!("there are more than one leader id, current: {}, new: {}", self.leader_id, request.leader_id);
            // TODO
        }

        // 比较log_index
        if request.prev_log_index > self.log.last_index() {
            warn!("reject append entries because prev_log_index {} is greater than last index {}", request.prev_log_index, self.log.last_index());
            return refuse_resp;
        }
        
        // 比较term
        if request.prev_log_index > self.log.start_index() {
            let log_entry = self.log.entry(request.prev_log_index).unwrap();
            if request.prev_log_term != log_entry.term {
                info!("reject append entries because prev_log_term {} is not equal to log entry term {}", request.prev_log_term, log_entry.term);
                return refuse_resp;
            }

        }
        
        // entries为空 => 心跳
        if request.entries.is_empty() {
            info!("receive heartbeat from leader {}", request.leader_id);
            self.follower_advance_commit_index(request.leader_commit);
            return proto::AppendEntriesResp {
                term: self.current_term,
                success: true,
            };
        }

        // 日志复制
        let mut entries_to_be_replicated: Vec<proto::LogEntry> = Vec::new();
        let mut index = request.prev_log_index;
        for entry in request.entries.iter() {
            index += 1;
            if entry.index != index {  // TODO 排序entries
                error!("request entries index is not incremental");
                return refuse_resp;
            }
            if index < self.log.start_index() {
                continue;
            }
            if self.log.last_index() >= index {
                let log_entry = self.log.entry(index).unwrap();
                if log_entry.term == entry.term {
                    continue;
                }
                // 删除冲突的日志
                info!("delete conflict log entry, index: {}, term: {}", index, log_entry.term);
                let last_index_kept = index - 1;
                self.log.truncate_suffix(last_index_kept);
            }
            entries_to_be_replicated.push(entry.clone());
        }

        let mut configuration_entries = Vec::new();
        for entry in entries_to_be_replicated.iter() {
            if entry.r#type() == proto::EntryType::Configuration {
                configuration_entries.push(entry.clone());
            }
        }

        // 更新日志
        self.log.append_entries(entries_to_be_replicated);

        // 应用配置
        for configuration_entry in configuration_entries.iter() {
            self.apply_configuration(config::Configuration::from_data(configuration_entry.data.as_ref()));
        }
        
        // 更新commit_index
        self.follower_advance_commit_index(request.leader_commit);

        proto::AppendEntriesResp {
            term: self.current_term,
            success: true,
        }
    }

    pub fn handle_request_vote(&mut self, request: &proto::RequestVoteReq) -> proto::RequestVoteResp {
        let refuse_reply = proto::RequestVoteResp {
            term: self.current_term,
            vote_granted: false,
        };

        // 比较任期
        if request.term < self.current_term {
            info!("Refuse vote for {} due to candidate's term is smaller", request.candidate_id);
            return refuse_reply;
        }

        // leader或candidate 收到term大于自己的candidate的RequestVote RPC => 回退到Follower
        if request.term > self.current_term {
            self.step_down(request.term);
        }

        // candidate日志是否最新
        let log_is_ok = request.term > self.log.last_term() || (request.term == self.log.last_term() && request.last_log_index >= self.log.last_index());
        if !log_is_ok {
            return refuse_reply;
        }

        // 已投给其他candidate
        if self.voted_for != config::NONE_SERVER_ID && self.voted_for != request.candidate_id {
            info!("Refuse vote for {} due to already voted for {}", request.candidate_id, self.voted_for);
            return refuse_reply;
        }

        match self.state {
            State::Leader => {
                panic!("leader {} receive request vote from {}", self.server_id, request.candidate_id);
            },
            State::Candidate => {
                panic!("candidate {} receive request vote from {}", self.server_id, request.candidate_id);
            },
            State::Follower => {
            }
        }

        // 投票给候选者
        info!("Agree vote for server id {}", request.candidate_id);
        self.voted_for = request.candidate_id;
        // 重置选举计时器
        self.election_timer.lock().unwrap().reset(util::rand_election_timeout());

        proto::RequestVoteResp {
            term: self.current_term,
            vote_granted: true
        }
    }


    pub fn handle_install_snapshot(&mut self, request: &proto::InstallSnapshotReq) -> proto::InstallSnapshotResp {
        info!("handle_install_snapshot");
        let reply = proto::InstallSnapshotResp {
            term: 1,
        };
        reply
    }

    pub fn handle_get_leader(&mut self, request: &proto::GetLeaderReq) -> proto::GetLeaderResp {
        if self.state == State::Leader {
            return proto::GetLeaderResp {
                leader: Some(proto::Server {
                    server_id: self.server_id,
                    server_addr: self.server_addr.clone(),
                })
            }
        }
        for peer in self.peer_manager.peers() {
            if peer.server_id == self.leader_id {
                return proto::GetLeaderResp {
                    leader: Some(proto::Server {
                        server_id: peer.server_id,
                        server_addr: peer.server_addr.clone(),
                    })
                }
            }
        }
        proto::GetLeaderResp {
            leader: None
        }
    }

    pub fn handle_get_configuration(&mut self, request: &proto::GetConfigurationReq) -> proto::GetConfigurationResp {
        let mut servers: Vec<proto::Server> = Vec::new();
        for peer in self.peer_manager.peers() {
            servers.push(proto::Server {
                server_id: peer.server_id,
                server_addr: peer.server_addr.clone(),
            })
        }
        servers.push(proto::Server {
            server_id: self.server_id,
            server_addr: self.server_addr.clone(),
        });

        let reply = proto::GetConfigurationResp {
            servers
        };
        reply
    }
    
    pub fn handle_set_configuration(&mut self, request: &proto::SetConfigurationReq) -> proto::SetConfigurationResp {
        let refuse_reply = proto::SetConfigurationResp { success: false };

        if request.new_servers.is_empty() {
            return refuse_reply;
        }

        let last_configuration =  self.log.last_configuration();
        // 最近一次Cold,new还没提交Cnew时，不能更新成员配置
        if !last_configuration.old_servers.is_empty() && !last_configuration.new_servers.is_empty(){
            return refuse_reply;
        }

        // TODO 新增成员catch up

        // 新增Cold,new日志条目
        let success = self.append_configuration(Some(request.new_servers.as_ref()));

        proto::SetConfigurationResp {
            success
        }
    }
}
