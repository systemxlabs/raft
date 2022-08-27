use log::info;
use crate::config;

#[derive(Debug)]
pub struct Peer {
    pub server_id: u64,
    pub server_addr: String,
    pub next_index: u64,
    pub match_index: u64,
    pub vote_granted: bool,
    pub configuration_state: config::ConfigurationState,
}

impl Peer {
    pub fn new(server_id: u64, server_addr: String) -> Self {
        Peer {
            server_id,
            server_addr,
            next_index: 1,  // TODO restore snapshot
            match_index: 0,
            vote_granted: false,
            configuration_state: config::ConfigurationState::new(),
        }
    }
}

#[derive(Debug)]
pub struct PeerManager {
    peers: Vec<Peer>
}

impl PeerManager {
    pub fn new() -> Self {
        PeerManager {
            peers: Vec::new()
        }
    }

    pub fn add_peers(&mut self, peers: Vec<Peer>) {
        self.peers.extend(peers);
    }
        
    pub fn peers_mut(&mut self) -> &mut Vec<Peer> {
        &mut self.peers
    }

    pub fn peers(&self) -> &Vec<Peer> {
        &self.peers
    }

    pub fn peers_num(&self) -> usize {
        self.peers.len()
    }

    pub fn peer_server_ids(&self) -> Vec<u64> {
        self.peers.iter().map(|peer| peer.server_id).collect()
    }

    pub fn peer(&mut self, server_id: u64) -> Option<&mut Peer> {
        self.peers.iter_mut().find(|peer| peer.server_id == server_id)
    }

    pub fn contains(&self, server_id: u64) -> bool {
        self.peers.iter().find(|peer| peer.server_id == server_id).is_some()
    }

    pub fn reset_vote(&mut self) {
        self.peers_mut().iter_mut().for_each(|peer| peer.vote_granted = false);
    }

    // 从match_index中找到多数的match_index
    pub fn quorum_match_index(&self, leader_configuration_state: &config::ConfigurationState, leader_last_index: u64) -> u64 {
        let mut new_match_indexes: Vec<u64> = Vec::new();
        if leader_configuration_state.in_new {
            new_match_indexes.push(leader_last_index);
        }
        for peer in self.peers.iter() {
            if peer.configuration_state.in_new {
                new_match_indexes.push(peer.match_index);
            }
        }
        new_match_indexes.sort();
        // new server 满足多数派的match index
        let new_quorum_match_index = {
            if new_match_indexes.len() == 0 {
                std::u64::MAX
            } else {
                new_match_indexes.get((new_match_indexes.len() - 1) / 2).unwrap().clone()
            }
        };

        let mut old_match_indexes: Vec<u64> = Vec::new();
        if leader_configuration_state.in_old {
            old_match_indexes.push(leader_last_index);
        }
        for peer in self.peers.iter() {
            if peer.configuration_state.in_old {
                old_match_indexes.push(peer.match_index);
            }
        }
        old_match_indexes.sort();
        // old server 满足多数派的match index
        let old_quorum_match_index = {
            if old_match_indexes.len() == 0 {
                std::u64::MAX
            } else {
                old_match_indexes.get((old_match_indexes.len() - 1) / 2).unwrap().clone()
            }
        };

        // 满足联合共识
        return std::cmp::min(new_quorum_match_index, old_quorum_match_index);
    }

    pub fn quorum_vote_granted(&self, leader_configuration_state: &config::ConfigurationState) -> bool {
        let mut total_new_servers = 0;
        let mut granted_new_servers = 0;

        let mut total_old_servers = 0;
        let mut granted_old_servers = 0;

        if leader_configuration_state.in_new {
            total_new_servers += 1;
            granted_new_servers += 1;
        }
        if leader_configuration_state.in_old {
            total_old_servers += 1;
            granted_old_servers += 1;
        }

        for peer in self.peers().iter() {
            if peer.configuration_state.in_new {
                total_new_servers += 1;
                if peer.vote_granted {
                    granted_new_servers += 1;
                }
            }
            if peer.configuration_state.in_old {
                total_old_servers += 1;
                if peer.vote_granted {
                    granted_old_servers += 1;
                }
            }
        }

        // 满足联合共识
        let new_servers_quorum = {
            total_new_servers == 0 || granted_new_servers > (total_new_servers / 2)
        };
        let old_servers_quorum = {
            total_old_servers == 0 || granted_old_servers > (total_old_servers / 2)
        };
        return new_servers_quorum && old_servers_quorum;
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn test_peers() {
        let mut peer_manager = super::PeerManager::new();
        let peer1 = super::Peer {
            server_id: 1,
            server_addr: "127.0.0.1:9001".to_string(),
            next_index: 3,
            match_index: 2,
            vote_granted: false,
            configuration_state: crate::config::ConfigurationState::new(),
        };
        let peer2 = super::Peer {
            server_id: 2,
            server_addr: "127.0.0.1:9002".to_string(),
            next_index: 2,
            match_index: 2,
            vote_granted: false,
            configuration_state: crate::config::ConfigurationState::new(),
        };
        peer_manager.add_peers(vec![peer1, peer2]);
        println!("{:?}", peer_manager);
        assert_eq!(peer_manager.peers().len(), 2);
    }
}
