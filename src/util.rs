use crate::config;
use rand::{self, Rng};
use std::time::{Duration, Instant};

pub fn rand_election_timeout() -> Duration {
    let mut rng = rand::thread_rng();
    let timeout =
        rng.gen_range(config::ELECTION_TIMEOUT_MIN_MILLIS..config::ELECTION_TIMEOUT_MAX_MILLIS);
    Duration::from_millis(timeout)
}
