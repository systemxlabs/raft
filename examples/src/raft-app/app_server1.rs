fn main () {
    println!("Hello, world!");
    raft::server::start("[::1]:9001");
}