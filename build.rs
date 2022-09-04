fn main() {
    tonic_build::configure()
        .type_attribute("LogEntry", "#[derive(serde::Deserialize, serde::Serialize)]")
        .type_attribute("Server", "#[derive(serde::Deserialize, serde::Serialize)]")
        .compile(&["proto/raft.proto"], &["proto"])
        .unwrap();
}