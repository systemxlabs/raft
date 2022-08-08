#[derive(Debug)]
pub struct Peer {
    pub server_id: u64,
    pub server_addr: String,
    pub next_index: u64,
    pub match_index: u64,
    pub vote_granted: bool,
}

impl Peer {
    pub fn new(server_id: u64, server_addr: String) -> Self {
        Peer {
            server_id,
            server_addr,
            next_index: 1,  // TODO
            match_index: 0,
            vote_granted: false,
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
        };
        let peer2 = super::Peer {
            server_id: 2,
            server_addr: "127.0.0.1:9002".to_string(),
            next_index: 2,
            match_index: 2,
            vote_granted: false,
        };
        peer_manager.add_peers(vec![peer1, peer2]);
        println!("{:?}", peer_manager);
        assert_eq!(peer_manager.peers().len(), 2);
    }
}
