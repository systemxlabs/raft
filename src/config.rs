use std::time::Duration;
use serde::{Deserialize, Serialize};
use crate::{peer, proto};


// 选举超时随机范围
pub const ELECTION_TIMEOUT_MAX_MILLIS: u64 = 15000;
pub const ELECTION_TIMEOUT_MIN_MILLIS: u64 = 10000;

// 心跳间隔时间
pub const HEARTBEAT_INTERVAL: Duration = Duration::from_millis(3000);

// 快照间隔时间
pub const SNAPSHOT_INTERVAL: Duration = Duration::from_millis(30000);

// 空server id
pub const NONE_SERVER_ID: u64 = 0;
// 空data
pub const NONE_DATA: &'static str = "None";

#[derive(Debug, PartialEq)]
pub struct ConfigurationState {
    pub in_new: bool,  // 在Cnew配置中，正常情况都处于Cnew
    pub in_old: bool,  // 在Cold配置中，成员变更期间部分会处于Cold
}

impl ConfigurationState {
    pub fn new() -> ConfigurationState {
        ConfigurationState { in_new: true, in_old: false }
    }
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct ServerInfo(pub u64, pub String);

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Configuration {
    pub old_servers: Vec<ServerInfo>,
    pub new_servers: Vec<ServerInfo>
}

impl Configuration {
    pub fn new() -> Configuration {
        Configuration { old_servers: Vec::new(), new_servers: Vec::new() }
    }
    pub fn from_data(data: &Vec<u8>) -> Configuration {
        bincode::deserialize(data).expect("Failed to convert vec<u8> to configuration")
    }
    pub fn to_data(&self) -> Vec<u8> {
        bincode::serialize(self).expect("Failed to convert configuration to vec<u8>")
    }
    pub fn append_new_servers(&mut self, new_servers: &Vec<proto::Server>) {
        for server in new_servers.iter() {
            self.new_servers.push(ServerInfo(server.server_id, server.server_addr.clone()));
        }
    }
    pub fn append_old_peers(&mut self, peers: &Vec<peer::Peer>) {
        for peer in peers.iter() {
            self.old_servers.push(ServerInfo(peer.server_id, peer.server_addr.clone()));
        }
    }
    pub fn gen_new_configuration(&self) -> Configuration {
        if self.old_servers.is_empty() || self.new_servers.is_empty() {
            panic!("Only Cold,new can generate Cnew");
        }
        Configuration { old_servers: Vec::new(), new_servers: self.new_servers.clone() }
    }
    pub fn query_configuration_state(&self, server_id: u64) -> ConfigurationState {
        ConfigurationState {
            in_new: self.new_servers.iter().find(|new_server| new_server.0 == server_id).is_some(),
            in_old: self.old_servers.iter().find(|old_server| old_server.0 == server_id).is_some(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::ConfigurationState;

    #[test]
    fn test_configuration() {
        let mut configuration = super::Configuration::new();
        configuration.old_servers.push(super::ServerInfo(1, "[::1]:9001".to_string()));
        configuration.new_servers.push(super::ServerInfo(2, "[::1]:9002".to_string()));

        let ser_data = configuration.to_data();
        let de_configuration = super::Configuration::from_data(&ser_data);

        assert_eq!(de_configuration, configuration);

        assert_eq!(configuration.query_configuration_state(1), ConfigurationState { in_new: false, in_old: true} );
    }
}