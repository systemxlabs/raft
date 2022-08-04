use std::time::Duration;

// 选举超时随机范围
pub const ELECTION_TIMEOUT_MAX_MILLIS: u64 = 15000;
pub const ELECTION_TIMEOUT_MIN_MILLIS: u64 = 10000;

// 心跳间隔时间
pub const HEARTBEAT_INTERVAL: Duration = Duration::from_millis(3000);

// 快照间隔时间
pub const SNAPSHOT_INTERVAL: Duration = Duration::from_millis(30000);