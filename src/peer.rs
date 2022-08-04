#[derive(Debug)]
pub struct Peer {
    pub server_id: u64,
    pub server_addr: String,
    pub next_index: u64,
    pub match_index: u64,
    pub vote_granted: bool,
    pub is_leader: bool,
}

impl Peer {
    pub fn new(server_id: u64, server_addr: String) -> Self {
        Peer {
            server_id,
            server_addr,
            next_index: 1,  // TODO
            match_index: 0,
            vote_granted: false,
            is_leader: false,
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
        
    pub fn peers(&mut self) -> &mut Vec<Peer> {
        &mut self.peers
    }

    pub fn leader(&mut self) -> Option<&mut Peer> {
        for peer in self.peers.iter_mut() {
            if peer.is_leader {
                return Some(peer);
            }
        }
        None
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
            is_leader: true,
        };
        let peer2 = super::Peer {
            server_id: 2,
            server_addr: "127.0.0.1:9002".to_string(),
            next_index: 2,
            match_index: 2,
            vote_granted: false,
            is_leader: false,
        };
        peer_manager.add_peers(vec![peer1, peer2]);
        println!("{:?}", peer_manager);
        assert_eq!(peer_manager.peers().len(), 2);
        assert_eq!(peer_manager.leader().unwrap().server_id, 1);
    }
}
