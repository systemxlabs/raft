fn main() {
    tonic_build::configure()
        .compile(&["proto/raft.proto"], &["proto"])
        .unwrap();
}